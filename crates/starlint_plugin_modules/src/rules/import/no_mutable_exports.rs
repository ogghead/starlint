//! Rule: `import/no-mutable-exports`
//!
//! Forbid using `let` or `var` in exported declarations.
//! Mutable exports can lead to confusing behavior since importers receive
//! a live binding that can change unexpectedly.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `export let` and `export var` declarations.
#[derive(Debug)]
pub struct NoMutableExports;

impl LintRule for NoMutableExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-mutable-exports".to_owned(),
            description: "Forbid using `let` or `var` in exported declarations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let findings: Vec<(u32, u32, &str, usize)> = {
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
                            // Keyword starts at "export " (7 chars) offset within the trimmed line
                            let leading_ws = line.len().saturating_sub(trimmed.len());
                            let kw_offset =
                                line_offset.saturating_add(leading_ws).saturating_add(7); // "export " = 7 chars
                            (start, end, keyword, kw_offset)
                        },
                    )
                })
                .collect()
        };

        for (start, end, keyword, kw_offset) in findings {
            let kw_start = u32::try_from(kw_offset).unwrap_or(0);
            let kw_end = kw_start.saturating_add(u32::try_from(keyword.len()).unwrap_or(3));
            let fix = FixBuilder::new(
                format!("Replace `{keyword}` with `const`"),
                FixKind::SuggestionFix,
            )
            .replace(Span::new(kw_start, kw_end), "const")
            .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-mutable-exports".to_owned(),
                message: format!(
                    "Exporting mutable binding using `{keyword}` — use `const` instead"
                ),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMutableExports)];
        lint_source(source, "test.js", &rules)
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
