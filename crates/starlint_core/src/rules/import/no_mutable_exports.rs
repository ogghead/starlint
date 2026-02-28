//! Rule: `import/no-mutable-exports`
//!
//! Forbid using `let` or `var` in exported declarations.
//! Mutable exports can lead to confusing behavior since importers receive
//! a live binding that can change unexpectedly.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `export let` and `export var` declarations.
#[derive(Debug)]
pub struct NoMutableExports;

impl NativeRule for NoMutableExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-mutable-exports".to_owned(),
            description: "Forbid using `let` or `var` in exported declarations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings: Vec<(u32, u32, &str)> = {
            let source = ctx.source_text();
            source
                .lines()
                .enumerate()
                .filter_map(|(idx, line)| {
                    let trimmed = line.trim();
                    (trimmed.starts_with("export let ") || trimmed.starts_with("export var ")).then(
                        || {
                            let line_offset = source
                                .lines()
                                .take(idx)
                                .map(|l| l.len().saturating_add(1))
                                .sum::<usize>();
                            let start = u32::try_from(line_offset).unwrap_or(0);
                            let end = u32::try_from(line_offset.saturating_add(trimmed.len()))
                                .unwrap_or(0);
                            let keyword = if trimmed.starts_with("export let") {
                                "let"
                            } else {
                                "var"
                            };
                            (start, end, keyword)
                        },
                    )
                })
                .collect()
        };

        for (start, end, keyword) in findings {
            ctx.report_warning(
                "import/no-mutable-exports",
                &format!("Exporting mutable binding using `{keyword}` — use `const` instead"),
                Span::new(start, end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMutableExports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_export_let() {
        let diags = lint("export let count = 0;");
        assert_eq!(diags.len(), 1, "export let should be flagged");
    }

    #[test]
    fn test_flags_export_var() {
        let diags = lint("export var name = 'hello';");
        assert_eq!(diags.len(), 1, "export var should be flagged");
    }

    #[test]
    fn test_allows_export_const() {
        let diags = lint("export const value = 42;");
        assert!(diags.is_empty(), "export const should not be flagged");
    }
}
