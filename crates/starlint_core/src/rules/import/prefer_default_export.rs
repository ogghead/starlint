//! Rule: `import/prefer-default-export`
//!
//! Prefer a default export when a module only has a single export.
//! Single named exports are harder to rename and don't benefit from
//! the convenience of default import syntax.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags modules with a single named export and no default export.
#[derive(Debug)]
pub struct PreferDefaultExport;

impl NativeRule for PreferDefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/prefer-default-export".to_owned(),
            description: "Prefer a default export when a module only has a single export"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let finding: Option<(u32, u32)> = {
            let source = ctx.source_text();

            if source.contains("export default ") {
                return;
            }

            // Count named export statements (approximate text scan)
            let named_export_count = source
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    (trimmed.starts_with("export ")
                        && !trimmed.starts_with("export default ")
                        && !trimmed.starts_with("export type ")
                        && !trimmed.starts_with("export interface "))
                        || trimmed.starts_with("export{")
                })
                .count();

            if named_export_count == 1 {
                // Find the line with the single export for span reporting
                source.lines().enumerate().find_map(|(idx, line)| {
                    let trimmed = line.trim();
                    let is_named_export = trimmed.starts_with("export ")
                        && !trimmed.starts_with("export default ")
                        && !trimmed.starts_with("export type ")
                        && !trimmed.starts_with("export interface ");

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
            } else {
                None
            }
        };

        if let Some((start, end)) = finding {
            ctx.report(Diagnostic {
                rule_name: "import/prefer-default-export".to_owned(),
                message: "Prefer default export when there is only a single export".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDefaultExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_single_named_export() {
        let diags = lint("export const foo = 1;");
        assert_eq!(diags.len(), 1, "single named export should be flagged");
    }

    #[test]
    fn test_allows_multiple_named_exports() {
        let diags = lint("export const foo = 1;\nexport const bar = 2;");
        assert!(
            diags.is_empty(),
            "multiple named exports should not be flagged"
        );
    }

    #[test]
    fn test_allows_default_export() {
        let diags = lint("export default function foo() {}");
        assert!(diags.is_empty(), "default export should not be flagged");
    }
}
