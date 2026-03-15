//! Rule: `typescript/no-implied-eval`
//!
//! Disallow implied `eval()` usage. Flags calls to `setTimeout` and
//! `setInterval` where the first argument is a string literal (which gets
//! evaluated as code), and `new Function()` with a string literal body.
//!
//! Simplified syntax-only version — full checking requires type information.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-implied-eval";

/// Functions that perform implied eval when given a string argument.
const TIMER_FUNCTIONS: &[&str] = &["setTimeout", "setInterval"];

/// Flags `setTimeout(string)`, `setInterval(string)`, and `new Function(string)`.
#[derive(Debug)]
pub struct NoImpliedEval;

impl LintRule for NoImpliedEval {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow implied `eval()` usage".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression, AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::CallExpression(call) => {
                let callee_name: Option<String> = match ctx.node(call.callee) {
                    Some(AstNode::IdentifierReference(id)) => Some(id.name.clone()),
                    Some(AstNode::StaticMemberExpression(member)) => {
                        // Handle `window.setTimeout(...)` and `globalThis.setInterval(...)`
                        let is_global_object = matches!(
                            ctx.node(member.object),
                            Some(AstNode::IdentifierReference(id))
                                if id.name.as_str() == "window"
                                    || id.name.as_str() == "globalThis"
                        );
                        is_global_object.then(|| member.property.clone())
                    }
                    _ => None,
                };

                let Some(name) = callee_name else {
                    return;
                };

                if !TIMER_FUNCTIONS.contains(&name.as_str()) {
                    return;
                }

                // Flag if the first argument is a string literal.
                if call.arguments.first().is_some_and(|arg_id| {
                    matches!(ctx.node(*arg_id), Some(AstNode::StringLiteral(_)))
                }) {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "Implied `eval()` — do not pass a string to `{name}()`, use a function instead"
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::NewExpression(new_expr) => {
                // Flag `new Function("string body")`
                let is_function_constructor = matches!(
                    ctx.node(new_expr.callee),
                    Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Function"
                );

                if is_function_constructor
                    && new_expr.arguments.last().is_some_and(|arg_id| {
                        matches!(ctx.node(*arg_id), Some(AstNode::StringLiteral(_)))
                    })
                {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Implied `eval()` — do not use the `Function` constructor with a string body".to_owned(),
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoImpliedEval, "test.ts");

    #[test]
    fn test_flags_set_timeout_with_string() {
        let diags = lint("setTimeout(\"alert('hi')\", 100);");
        assert_eq!(
            diags.len(),
            1,
            "setTimeout with string arg should be flagged"
        );
    }

    #[test]
    fn test_flags_set_interval_with_string() {
        let diags = lint("setInterval(\"doStuff()\", 1000);");
        assert_eq!(
            diags.len(),
            1,
            "setInterval with string arg should be flagged"
        );
    }

    #[test]
    fn test_flags_new_function_with_string() {
        let diags = lint("var f = new Function(\"return 1\");");
        assert_eq!(
            diags.len(),
            1,
            "new Function with string arg should be flagged"
        );
    }

    #[test]
    fn test_allows_set_timeout_with_function() {
        let diags = lint("setTimeout(() => {}, 100);");
        assert!(
            diags.is_empty(),
            "setTimeout with function arg should not be flagged"
        );
    }

    #[test]
    fn test_allows_set_interval_with_function() {
        let diags = lint("setInterval(function() {}, 1000);");
        assert!(
            diags.is_empty(),
            "setInterval with function arg should not be flagged"
        );
    }

    #[test]
    fn test_flags_window_set_timeout_with_string() {
        let diags = lint("window.setTimeout(\"alert('hi')\", 100);");
        assert_eq!(
            diags.len(),
            1,
            "window.setTimeout with string arg should be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_call() {
        let diags = lint("console.log(\"hello\");");
        assert!(
            diags.is_empty(),
            "unrelated call with string arg should not be flagged"
        );
    }
}
