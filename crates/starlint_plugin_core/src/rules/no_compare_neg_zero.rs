//! Rule: `no-compare-neg-zero`
//!
//! Disallow comparing against `-0`. The expression `x === -0` does not
//! work as expected because `-0 === 0` is `true` in JavaScript.
//! Use `Object.is(x, -0)` instead.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags comparisons against `-0`.
#[derive(Debug)]
pub struct NoCompareNegZero;

impl LintRule for NoCompareNegZero {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-compare-neg-zero".to_owned(),
            description: "Disallow comparing against `-0`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        // Only flag comparison operators
        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        let left_neg_zero = is_negative_zero(ctx, expr.left);
        let right_neg_zero = is_negative_zero(ctx, expr.right);

        if !left_neg_zero && !right_neg_zero {
            return;
        }

        // Fix: `x === -0` → `Object.is(x, -0)`
        let value_id = if right_neg_zero {
            expr.left
        } else {
            expr.right
        };

        let fix = ctx.node(value_id).and_then(|val_node| {
            let val_span = val_node.span();
            let source = ctx.source_text();
            source
                .get(val_span.start as usize..val_span.end as usize)
                .map(|val_text| {
                    let replacement = format!("Object.is({val_text}, -0)");
                    Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr.span.start, expr.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                })
        });

        ctx.report(Diagnostic {
            rule_name: "no-compare-neg-zero".to_owned(),
            message: format!(
                "Do not use the `{}` operator to compare against `-0`",
                expr.operator.as_str()
            ),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Error,
            help: Some("Use `Object.is(x, -0)` to test for negative zero".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

/// Check if a node is the literal `-0` (`UnaryExpression(-)` → `NumericLiteral(0)`).
fn is_negative_zero(ctx: &LintContext<'_>, id: NodeId) -> bool {
    let Some(AstNode::UnaryExpression(unary)) = ctx.node(id) else {
        return false;
    };
    if unary.operator != UnaryOperator::UnaryNegation {
        return false;
    }
    matches!(
        ctx.node(unary.argument),
        Some(AstNode::NumericLiteral(lit)) if lit.value == 0.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoCompareNegZero);

    #[test]
    fn test_flags_strict_equality_neg_zero() {
        let diags = lint("if (x === -0) {}");
        assert_eq!(diags.len(), 1, "=== -0 should be flagged");
    }

    #[test]
    fn test_flags_loose_equality_neg_zero() {
        let diags = lint("if (x == -0) {}");
        assert_eq!(diags.len(), 1, "== -0 should be flagged");
    }

    #[test]
    fn test_flags_inequality_neg_zero() {
        let diags = lint("if (x !== -0) {}");
        assert_eq!(diags.len(), 1, "!== -0 should be flagged");
    }

    #[test]
    fn test_flags_less_than_neg_zero() {
        let diags = lint("if (x < -0) {}");
        assert_eq!(diags.len(), 1, "< -0 should be flagged");
    }

    #[test]
    fn test_flags_neg_zero_on_left() {
        let diags = lint("if (-0 === x) {}");
        assert_eq!(diags.len(), 1, "-0 on left side should be flagged");
    }

    #[test]
    fn test_allows_comparison_with_zero() {
        let diags = lint("if (x === 0) {}");
        assert!(diags.is_empty(), "comparison with 0 should not be flagged");
    }

    #[test]
    fn test_allows_comparison_with_number() {
        let diags = lint("if (x === -1) {}");
        assert!(diags.is_empty(), "comparison with -1 should not be flagged");
    }

    #[test]
    fn test_allows_arithmetic() {
        let diags = lint("const y = x + -0;");
        assert!(diags.is_empty(), "arithmetic with -0 should not be flagged");
    }
}
