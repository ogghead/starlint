//! Rule: `erasing-op` (OXC)
//!
//! Detect operations that always produce a known constant regardless of the
//! other operand, such as `x * 0`, `x & 0`, `x % 1`, or `x ** 0`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags operations that erase the value of one operand.
#[derive(Debug)]
pub struct ErasingOp;

impl LintRule for ErasingOp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "erasing-op".to_owned(),
            description: "Detect operations that always produce a known constant".to_owned(),
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

        let msg = match expr.operator {
            // x * 0 or 0 * x → always 0
            BinaryOperator::Multiplication => (is_integer_literal(ctx, expr.left, 0)
                || is_integer_literal(ctx, expr.right, 0))
            .then_some("Multiplying by 0 always produces 0"),
            // x & 0 or 0 & x → always 0
            BinaryOperator::BitwiseAnd => (is_integer_literal(ctx, expr.left, 0)
                || is_integer_literal(ctx, expr.right, 0))
            .then_some("Bitwise AND with 0 always produces 0"),
            // x % 1 → always 0
            BinaryOperator::Remainder => {
                is_integer_literal(ctx, expr.right, 1).then_some("Remainder by 1 always produces 0")
            }
            // x ** 0 → always 1
            BinaryOperator::Exponential => is_integer_literal(ctx, expr.right, 0)
                .then_some("Exponentiation by 0 always produces 1"),
            _ => None,
        };

        if let Some(message) = msg {
            let replacement = match expr.operator {
                BinaryOperator::Exponential => "1",
                _ => "0",
            };
            ctx.report(Diagnostic {
                rule_name: "erasing-op".to_owned(),
                message: message.to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some(format!(
                    "This expression always evaluates to `{replacement}`"
                )),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: replacement.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if a node is a numeric literal with the given integer value.
fn is_integer_literal(ctx: &LintContext<'_>, id: NodeId, value: u64) -> bool {
    match ctx.node(id) {
        Some(AstNode::NumericLiteral(n)) => {
            let expected = format!("{value}");
            n.raw.as_str() == expected
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(ErasingOp);

    #[test]
    fn test_flags_multiply_by_zero() {
        let diags = lint("var n = x * 0;");
        assert_eq!(diags.len(), 1, "x * 0 should be flagged");
    }

    #[test]
    fn test_flags_zero_multiply() {
        let diags = lint("var n = 0 * x;");
        assert_eq!(diags.len(), 1, "0 * x should be flagged");
    }

    #[test]
    fn test_flags_bitwise_and_zero() {
        let diags = lint("var n = x & 0;");
        assert_eq!(diags.len(), 1, "x & 0 should be flagged");
    }

    #[test]
    fn test_flags_remainder_by_one() {
        let diags = lint("var n = x % 1;");
        assert_eq!(diags.len(), 1, "x % 1 should be flagged");
    }

    #[test]
    fn test_flags_exponent_zero() {
        let diags = lint("var n = x ** 0;");
        assert_eq!(diags.len(), 1, "x ** 0 should be flagged");
    }

    #[test]
    fn test_allows_normal_multiplication() {
        let diags = lint("var n = x * 2;");
        assert!(
            diags.is_empty(),
            "normal multiplication should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_remainder() {
        let diags = lint("var n = x % 3;");
        assert!(diags.is_empty(), "normal remainder should not be flagged");
    }
}
