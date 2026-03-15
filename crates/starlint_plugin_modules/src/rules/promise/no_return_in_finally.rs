//! Rule: `promise/no-return-in-finally`
//!
//! Forbid `return` statements in `.finally()` callbacks. Returning from
//! `.finally()` silently swallows the resolved/rejected value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.finally()` callbacks that contain `return` statements.
///
/// Heuristic: scans the source text of `.finally()` callback arguments
/// for `return` keywords.
#[derive(Debug)]
pub struct NoReturnInFinally;

impl LintRule for NoReturnInFinally {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-return-in-finally".to_owned(),
            description: "Forbid `return` in `.finally()` callbacks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "finally" {
            return;
        }

        // Check the first argument (the finally callback)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let Some(arg_expr) = ctx.node(*first_arg) else {
            return;
        };

        if matches!(arg_expr, AstNode::SpreadElement(_)) {
            return;
        }

        // Expression arrows don't have explicit return, skip
        if let AstNode::ArrowFunctionExpression(arrow) = arg_expr {
            if arrow.expression {
                return;
            }
        }

        let start = usize::try_from(arg_expr.span().start).unwrap_or(0);
        let end = usize::try_from(arg_expr.span().end).unwrap_or(0);
        let body_text = ctx.source_text().get(start..end).unwrap_or_default();

        // Heuristic: check for return statement in the body
        // Skip `return;` (empty return) which is less harmful
        if body_text.contains("return ") {
            ctx.report(Diagnostic {
                rule_name: "promise/no-return-in-finally".to_owned(),
                message: "Do not use `return` with a value in `.finally()` — it silently swallows the promise result".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
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

    starlint_rule_framework::lint_rule_test!(NoReturnInFinally);

    #[test]
    fn test_flags_return_in_finally() {
        let diags = lint("p.finally(() => { return 42; });");
        assert_eq!(diags.len(), 1, "should flag return in .finally()");
    }

    #[test]
    fn test_allows_finally_without_return() {
        let diags = lint("p.finally(() => { cleanup(); });");
        assert!(
            diags.is_empty(),
            ".finally() without return should be allowed"
        );
    }

    #[test]
    fn test_allows_expression_arrow_finally() {
        let diags = lint("p.finally(() => cleanup());");
        assert!(
            diags.is_empty(),
            "expression arrow in .finally() should be allowed"
        );
    }
}
