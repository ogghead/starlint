//! Rule: `no-negation-in-equality-check`
//!
//! Disallow negation in the left-hand side of equality checks. Expressions
//! like `!a == b` or `!a === b` are parsed as `(!a) == b`, not `a != b`.
//! This is almost always a mistake and leads to confusing behavior.

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `!a == b` and `!a === b` patterns where negation binds tighter
/// than the equality operator.
#[derive(Debug)]
pub struct NoNegationInEqualityCheck;

impl LintRule for NoNegationInEqualityCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-negation-in-equality-check".to_owned(),
            description: "Disallow negation in the left-hand side of equality checks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        // Only check `==` and `===` operators
        if expr.operator != BinaryOperator::Equality
            && expr.operator != BinaryOperator::StrictEquality
        {
            return;
        }

        // Check if the left side is a `!` unary expression
        if let Some(AstNode::UnaryExpression(unary)) = ctx.node(expr.left) {
            if unary.operator == UnaryOperator::LogicalNot {
                let op_str = if expr.operator == BinaryOperator::Equality {
                    "=="
                } else {
                    "==="
                };
                let negated_op = if expr.operator == BinaryOperator::Equality {
                    "!="
                } else {
                    "!=="
                };

                // Fix: `!a == b` → `a != b`, `!a === b` → `a !== b`
                let expr_span = expr.span;
                let unary_arg = unary.argument;
                #[allow(clippy::as_conversions)]
                let fix = {
                    let source = ctx.source_text();
                    let inner_span = ctx.node(unary_arg).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let right_span = ctx.node(expr.right).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let inner_text = source
                        .get(inner_span.start as usize..inner_span.end as usize)
                        .unwrap_or("");
                    let right_text = source
                        .get(right_span.start as usize..right_span.end as usize)
                        .unwrap_or("");
                    let replacement = format!("{inner_text} {negated_op} {right_text}");
                    Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr_span.start, expr_span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                };

                ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                    rule_name: "no-negation-in-equality-check".to_owned(),
                    message: format!(
                        "Negation in left-hand side of `{op_str}` is confusing — `!a {op_str} b` is parsed as `(!a) {op_str} b`"
                    ),
                    span: Span::new(expr_span.start, expr_span.end),
                    severity: Severity::Warning,
                    help: Some(format!(
                        "Use `a {negated_op} b` instead, or wrap in parentheses: `(!a) {op_str} b`"
                    )),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNegationInEqualityCheck);

    #[test]
    fn test_flags_negation_loose_equality() {
        let diags = lint("if (!a == b) {}");
        assert_eq!(diags.len(), 1, "!a == b should be flagged");
    }

    #[test]
    fn test_flags_negation_strict_equality() {
        let diags = lint("if (!a === b) {}");
        assert_eq!(diags.len(), 1, "!a === b should be flagged");
    }

    #[test]
    fn test_allows_inequality() {
        let diags = lint("if (a != b) {}");
        assert!(diags.is_empty(), "a != b should not be flagged");
    }

    #[test]
    fn test_allows_strict_inequality() {
        let diags = lint("if (a !== b) {}");
        assert!(diags.is_empty(), "a !== b should not be flagged");
    }

    #[test]
    fn test_allows_standalone_negation() {
        let diags = lint("if (!a) {}");
        assert!(
            diags.is_empty(),
            "standalone negation should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_equality() {
        let diags = lint("if (a == b) {}");
        assert!(diags.is_empty(), "normal equality should not be flagged");
    }
}
