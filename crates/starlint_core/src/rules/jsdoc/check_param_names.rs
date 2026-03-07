//! Rule: `jsdoc/check-param-names`
//!
//! Enforce `@param` names match function parameters.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

#[derive(Debug)]
pub struct CheckParamNames;

/// Extract `@param` names from a `JSDoc` block.
fn extract_param_names(block: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in block.lines() {
        let trimmed = super::trim_jsdoc_line(line);
        if let Some(tag_rest) = trimmed.strip_prefix("@param") {
            let tag_content = tag_rest.trim();
            // Skip optional type annotation `{...}`
            let after_type = if tag_content.starts_with('{') {
                tag_content
                    .find('}')
                    .and_then(|i| tag_content.get(i.saturating_add(1)..))
                    .unwrap_or_default()
                    .trim()
            } else {
                tag_content
            };
            // First word after type is the param name
            if let Some(name) = after_type.split_whitespace().next() {
                // Strip optional brackets `[name]` or `[name=default]`
                let clean = name
                    .trim_start_matches('[')
                    .split('=')
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches(']');
                if !clean.is_empty() {
                    names.push(clean.to_owned());
                }
            }
        }
    }
    names
}

/// Extract function parameter names from a function signature line.
fn extract_fn_params(source: &str, search_after: usize) -> Vec<String> {
    let remaining = source.get(search_after..).unwrap_or_default();
    // Find the next function-like construct
    let fn_start = remaining
        .find("function ")
        .or_else(|| remaining.find("function("))
        .or_else(|| remaining.find("=>"));

    if let Some(offset) = fn_start {
        let from_fn = remaining.get(offset..).unwrap_or_default();
        if let Some(paren_start) = from_fn.find('(') {
            if let Some(paren_end) = from_fn.get(paren_start..).and_then(|s| s.find(')')) {
                let params_str = from_fn
                    .get(paren_start.saturating_add(1)..paren_start.saturating_add(paren_end))
                    .unwrap_or_default();
                return params_str
                    .split(',')
                    .filter_map(|p| {
                        let name = p
                            .trim()
                            .split(':')
                            .next()
                            .unwrap_or_default()
                            .split('=')
                            .next()
                            .unwrap_or_default()
                            .trim();
                        if name.is_empty() {
                            None
                        } else {
                            Some(name.to_owned())
                        }
                    })
                    .collect();
            }
        }
    }
    vec![]
}

impl LintRule for CheckParamNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/check-param-names".to_owned(),
            description: "Enforce `@param` names match function parameters".to_owned(),
            category: Category::Correctness,
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

                let doc_params = extract_param_names(block);
                if !doc_params.is_empty() {
                    let fn_params = extract_fn_params(&source, abs_end);
                    for doc_param in &doc_params {
                        if !fn_params.iter().any(|fp| fp == doc_param) {
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                            ctx.report(Diagnostic {
                                rule_name: "jsdoc/check-param-names".to_owned(),
                                message: format!(
                                    "`@param {doc_param}` does not match any function parameter"
                                ),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CheckParamNames)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_mismatched_param() {
        let source = "/** @param {string} wrong */\nfunction foo(name) {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_matching_param() {
        let source = "/** @param {string} name */\nfunction foo(name) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_params() {
        let source = "/** Some description */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
