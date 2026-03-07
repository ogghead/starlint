//! Rule: `jsdoc/no-defaults`
//!
//! Forbid `@default` and `@defaultvalue` tags in `JSDoc` comments.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

#[derive(Debug)]
pub struct NoDefaults;

impl LintRule for NoDefaults {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/no-defaults".to_owned(),
            description: "Forbid `@default` tags in JSDoc comments".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        let mut pos = 0;
        while let Some(start) = source.get(pos..).and_then(|s| s.find("/**")) {
            let abs_start = pos.saturating_add(start);
            let search_from = abs_start.saturating_add(3);
            if let Some(end) = source.get(search_from..).and_then(|s| s.find("*/")) {
                let abs_end = search_from.saturating_add(end).saturating_add(2);
                let block = source.get(abs_start..abs_end).unwrap_or_default();

                for line in block.lines() {
                    let trimmed = super::trim_jsdoc_line(line);
                    if trimmed.starts_with("@default") {
                        // Match @default or @defaultvalue but not @defaultsomethingelse
                        let after = trimmed.strip_prefix("@default").unwrap_or_default();
                        if after.is_empty() || after.starts_with(' ') || after.starts_with("value")
                        {
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                            ctx.report(Diagnostic {
                                rule_name: "jsdoc/no-defaults".to_owned(),
                                message: "Unexpected `@default` tag in JSDoc comment".to_owned(),
                                span: Span::new(span_start, span_end),
                                severity: Severity::Warning,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                }

                pos = abs_end;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDefaults)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_default_tag() {
        let source = "/** @default 42 */\nconst x = 42;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_defaultvalue_tag() {
        let source = "/** @defaultvalue 42 */\nconst x = 42;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_no_default_tag() {
        let source = "/** Some description */\nconst x = 42;";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
