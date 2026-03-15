//! Rule: `jsdoc/require-returns`
//!
//! Require `@returns` tag for functions with return statements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

#[derive(Debug)]
pub struct RequireReturns;

/// Check if a `JSDoc` block contains `@returns` or `@return`.
fn has_returns_tag(block: &str) -> bool {
    block.lines().any(|line| {
        let trimmed = super::trim_jsdoc_line(line);
        trimmed.starts_with("@returns") || trimmed.starts_with("@return ")
    })
}

/// Simple heuristic: check if the function body after the `JSDoc` contains a `return` statement
/// with a value (not just bare `return;`).
fn has_return_value(source: &str, after_pos: usize) -> bool {
    let remaining = source.get(after_pos..).unwrap_or_default();
    // Find the function body
    let fn_start = remaining
        .find("function ")
        .or_else(|| remaining.find("function("))
        .or_else(|| remaining.find("=>"));

    if let Some(offset) = fn_start {
        let from_fn = remaining.get(offset..).unwrap_or_default();
        // Find opening brace
        if let Some(brace_pos) = from_fn.find('{') {
            let body = from_fn.get(brace_pos..).unwrap_or_default();
            // Simple check: look for "return " followed by something other than ";"
            let mut search_pos = 0;
            while let Some(ret_pos) = body.get(search_pos..).and_then(|s| s.find("return")) {
                let abs_ret = search_pos.saturating_add(ret_pos);
                let after_return = body
                    .get(abs_ret.saturating_add(6)..)
                    .unwrap_or_default()
                    .trim_start();
                if !after_return.starts_with(';')
                    && !after_return.starts_with('}')
                    && !after_return.is_empty()
                {
                    return true;
                }
                search_pos = abs_ret.saturating_add(6);
            }
        }
        // Arrow function without braces (implicit return)
        if from_fn.contains("=>") {
            let arrow_pos = from_fn.find("=>").unwrap_or(0);
            let after_arrow = from_fn
                .get(arrow_pos.saturating_add(2)..)
                .unwrap_or_default()
                .trim_start();
            if !after_arrow.starts_with('{') && !after_arrow.is_empty() {
                return true;
            }
        }
    }
    false
}

impl LintRule for RequireReturns {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/require-returns".to_owned(),
            description: "Require `@returns` tag for functions with return values".to_owned(),
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

                if !has_returns_tag(block) && has_return_value(&source, abs_end) {
                    let span_start = u32::try_from(abs_start).unwrap_or(0);
                    let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                    ctx.report(Diagnostic {
                        rule_name: "jsdoc/require-returns".to_owned(),
                        message: "Missing `@returns` tag for function with return value".to_owned(),
                        span: Span::new(span_start, span_end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
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
    starlint_rule_framework::lint_rule_test!(RequireReturns);

    #[test]
    fn test_flags_missing_returns() {
        let source = "/** Does something */\nfunction foo() { return 42; }";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_with_returns_tag() {
        let source = "/** Does something\n * @returns {number} The result\n */\nfunction foo() { return 42; }";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_void_function() {
        let source = "/** Does something */\nfunction foo() { console.log(1); }";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
