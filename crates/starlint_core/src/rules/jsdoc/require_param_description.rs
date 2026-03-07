//! Rule: `jsdoc/require-param-description`
//!
//! Require `@param` tags have descriptions.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

#[derive(Debug)]
pub struct RequireParamDescription;

impl LintRule for RequireParamDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/require-param-description".to_owned(),
            description: "Require `@param` tags have descriptions".to_owned(),
            category: Category::Style,
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
                    if let Some(tag_rest) = trimmed.strip_prefix("@param") {
                        let tag_content = tag_rest.trim();
                        // Skip type annotation
                        let after_type = if tag_content.starts_with('{') {
                            tag_content
                                .find('}')
                                .and_then(|i| tag_content.get(i.saturating_add(1)..))
                                .unwrap_or_default()
                                .trim()
                        } else {
                            tag_content
                        };
                        // Skip the name (first word) and check if there's a description
                        let mut words = after_type.split_whitespace();
                        let _name = words.next(); // param name
                        let description: String = words.collect::<Vec<_>>().join(" ");
                        if description.trim().is_empty() {
                            // Extract the param name for the message
                            let param_name = after_type
                                .split_whitespace()
                                .next()
                                .unwrap_or_default()
                                .trim_start_matches('[')
                                .split('=')
                                .next()
                                .unwrap_or_default()
                                .trim_end_matches(']');
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                            ctx.report(Diagnostic {
                                rule_name: "jsdoc/require-param-description".to_owned(),
                                message: format!("`@param {param_name}` is missing a description"),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireParamDescription)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_param_without_description() {
        let source = "/** @param {string} x */\nfunction foo(x) {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_param_with_description() {
        let source = "/** @param {string} x The input value */\nfunction foo(x) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_param_name_only() {
        let source = "/** @param x */\nfunction foo(x) {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }
}
