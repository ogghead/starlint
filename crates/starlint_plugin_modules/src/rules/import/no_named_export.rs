//! Rule: `import/no-named-export`
//!
//! Forbid named exports. Some teams prefer a single default export per
//! module for consistency or to enforce a particular module pattern.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags named export statements.
#[derive(Debug)]
pub struct NoNamedExport;

impl LintRule for NoNamedExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-export".to_owned(),
            description: "Forbid named exports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("export ")
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let findings: Vec<(u32, u32)> = {
            let source = ctx.source_text();
            let mut found = Vec::new();
            let mut byte_offset: usize = 0;

            for line in source.lines() {
                let trimmed = line.trim();

                // Skip default exports and re-exports
                if !trimmed.starts_with("export default ") {
                    let is_named_export = trimmed.starts_with("export {")
                        || trimmed.starts_with("export const ")
                        || trimmed.starts_with("export function ")
                        || trimmed.starts_with("export class ")
                        || trimmed.starts_with("export let ")
                        || trimmed.starts_with("export var ")
                        || trimmed.starts_with("export enum ")
                        || trimmed.starts_with("export interface ")
                        || trimmed.starts_with("export type ")
                        || trimmed.starts_with("export async function ");

                    if is_named_export {
                        let start = u32::try_from(byte_offset).unwrap_or(0);
                        let end =
                            u32::try_from(byte_offset.saturating_add(trimmed.len())).unwrap_or(0);
                        found.push((start, end));
                    }
                }

                byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
            }
            found
        };

        for (start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "import/no-named-export".to_owned(),
                message: "Named exports are not allowed — use a default export instead".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNamedExport)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_named_export_block() {
        let diags = lint("const foo = 1;\nexport { foo };");
        assert_eq!(diags.len(), 1, "named export block should be flagged");
    }

    #[test]
    fn test_flags_export_const() {
        let diags = lint("export const foo = 1;");
        assert_eq!(diags.len(), 1, "export const should be flagged");
    }

    #[test]
    fn test_allows_default_export() {
        let diags = lint("export default function foo() {}");
        assert!(diags.is_empty(), "default export should not be flagged");
    }
}
