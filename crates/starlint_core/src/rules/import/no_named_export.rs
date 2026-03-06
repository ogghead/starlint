//! Rule: `import/no-named-export`
//!
//! Forbid named exports. Some teams prefer a single default export per
//! module for consistency or to enforce a particular module pattern.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags named export statements.
#[derive(Debug)]
pub struct NoNamedExport;

impl NativeRule for NoNamedExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-export".to_owned(),
            description: "Forbid named exports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings: Vec<(u32, u32)> = {
            let source = ctx.source_text();
            source
                .lines()
                .enumerate()
                .filter_map(|(idx, line)| {
                    let trimmed = line.trim();

                    // Skip default exports and re-exports
                    if trimmed.starts_with("export default ") {
                        return None;
                    }

                    // Detect named exports
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

                    is_named_export.then(|| {
                        let line_offset = source
                            .lines()
                            .take(idx)
                            .map(|l| l.len().saturating_add(1))
                            .sum::<usize>();
                        let start = u32::try_from(line_offset).unwrap_or(0);
                        let end =
                            u32::try_from(line_offset.saturating_add(trimmed.len())).unwrap_or(0);
                        (start, end)
                    })
                })
                .collect()
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNamedExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
