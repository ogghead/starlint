//! Rule: `const-comparisons` (OXC)
//!
//! Detect comparisons that are always true or always false due to constant
//! operands. For example, `x > 5 && x > 3` (the second check is redundant)
//! or `x > 5 && x < 3` (always false — impossible range).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, LogicalOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags always-true or always-false constant comparisons.
#[derive(Debug)]
pub struct ConstComparisons;

impl LintRule for ConstComparisons {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "const-comparisons".to_owned(),
            description: "Detect always-true or always-false constant comparisons".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::LogicalExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::LogicalExpression(logical) = node else {
            return;
        };

        let Some(AstNode::BinaryExpression(left)) = ctx.node(logical.left) else {
            return;
        };
        let Some(AstNode::BinaryExpression(right)) = ctx.node(logical.right) else {
            return;
        };

        // Both sides must have one variable operand and one numeric literal
        let source = ctx.source_text();
        let left_info = extract_comparison_info(ctx, left, source);
        let right_info = extract_comparison_info(ctx, right, source);

        let (Some(l_info), Some(r_info)) = (left_info, right_info) else {
            return;
        };

        // The variable must be the same
        if l_info.var_text != r_info.var_text {
            return;
        }

        let finding = match logical.operator {
            // x > 5 && x < 3 → impossible (always false if bounds don't overlap)
            LogicalOperator::And => {
                check_impossible_range(l_info.op, l_info.value, r_info.op, r_info.value)
            }
            // x > 5 || x < 10 → always true (covers everything)
            LogicalOperator::Or => {
                check_tautological_range(l_info.op, l_info.value, r_info.op, r_info.value)
            }
            LogicalOperator::Coalesce => None,
        };

        if let Some(message) = finding {
            // Fix: replace always-false with `false`, always-true with `true`
            let is_always_false = message.contains("always false");
            let replacement = if is_always_false {
                "false".to_owned()
            } else {
                "true".to_owned()
            };
            let fix = Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(logical.span.start, logical.span.end),
                    replacement,
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "const-comparisons".to_owned(),
                message: message.to_owned(),
                span: Span::new(logical.span.start, logical.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Information about a comparison with one variable and one constant.
struct ComparisonInfo<'a> {
    /// The variable's source text.
    var_text: &'a str,
    /// The normalized comparison operator (always expressed as "variable OP value").
    op: BinaryOperator,
    /// The constant numeric value.
    value: f64,
}

/// Extract comparison info if one side is a variable and the other is a number.
fn extract_comparison_info<'s>(
    ctx: &LintContext<'_>,
    expr: &starlint_ast::node::BinaryExpressionNode,
    source: &'s str,
) -> Option<ComparisonInfo<'s>> {
    if !expr.operator.is_compare() {
        return None;
    }

    let left_node = ctx.node(expr.left)?;
    let right_node = ctx.node(expr.right)?;
    let left_num = get_numeric_value(left_node);
    let right_num = get_numeric_value(right_node);

    match (left_num, right_num) {
        // variable OP number
        (None, Some(value)) => {
            let left_span = left_node.span();
            let start = usize::try_from(left_span.start).ok()?;
            let end = usize::try_from(left_span.end).ok()?;
            let var_text = source.get(start..end)?;
            Some(ComparisonInfo {
                var_text,
                op: expr.operator,
                value,
            })
        }
        // number OP variable → flip the operator
        (Some(value), None) => {
            let right_span = right_node.span();
            let start = usize::try_from(right_span.start).ok()?;
            let end = usize::try_from(right_span.end).ok()?;
            let var_text = source.get(start..end)?;
            Some(ComparisonInfo {
                var_text,
                op: flip_comparison(expr.operator),
                value,
            })
        }
        _ => None,
    }
}

/// Get the numeric value from an `AstNode` if it's a numeric literal.
const fn get_numeric_value(node: &AstNode) -> Option<f64> {
    match node {
        AstNode::NumericLiteral(n) => Some(n.value),
        _ => None,
    }
}

/// Flip a comparison operator (e.g., `>` becomes `<`).
const fn flip_comparison(op: BinaryOperator) -> BinaryOperator {
    match op {
        BinaryOperator::LessThan => BinaryOperator::GreaterThan,
        BinaryOperator::GreaterThan => BinaryOperator::LessThan,
        BinaryOperator::LessEqualThan => BinaryOperator::GreaterEqualThan,
        BinaryOperator::GreaterEqualThan => BinaryOperator::LessEqualThan,
        other => other,
    }
}

/// Check if an AND combination is impossible (always false).
#[allow(clippy::float_cmp)]
fn check_impossible_range(
    left_op: BinaryOperator,
    left_val: f64,
    right_op: BinaryOperator,
    right_val: f64,
) -> Option<&'static str> {
    let impossible = match (left_op, right_op) {
        (
            BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan,
            BinaryOperator::LessThan,
        )
        | (BinaryOperator::GreaterThan, BinaryOperator::LessEqualThan) => left_val >= right_val,
        (BinaryOperator::GreaterEqualThan, BinaryOperator::LessEqualThan) => left_val > right_val,
        (BinaryOperator::LessThan | BinaryOperator::LessEqualThan, BinaryOperator::GreaterThan)
        | (BinaryOperator::LessThan, BinaryOperator::GreaterEqualThan) => right_val >= left_val,
        (BinaryOperator::LessEqualThan, BinaryOperator::GreaterEqualThan) => right_val > left_val,
        _ => false,
    };

    impossible.then_some("This comparison is always false — the range is impossible")
}

/// Check if an OR combination is tautological (always true).
#[allow(clippy::float_cmp)]
fn check_tautological_range(
    left_op: BinaryOperator,
    left_val: f64,
    right_op: BinaryOperator,
    right_val: f64,
) -> Option<&'static str> {
    let tautological = match (left_op, right_op) {
        (
            BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan,
            BinaryOperator::LessThan,
        )
        | (BinaryOperator::GreaterThan, BinaryOperator::LessEqualThan) => right_val > left_val,
        (BinaryOperator::GreaterEqualThan, BinaryOperator::LessEqualThan) => right_val >= left_val,
        (BinaryOperator::LessThan | BinaryOperator::LessEqualThan, BinaryOperator::GreaterThan)
        | (BinaryOperator::LessThan, BinaryOperator::GreaterEqualThan) => left_val > right_val,
        (BinaryOperator::LessEqualThan, BinaryOperator::GreaterEqualThan) => left_val >= right_val,
        _ => false,
    };

    tautological.then_some("This comparison is always true — the condition is tautological")
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(ConstComparisons);

    #[test]
    fn test_flags_impossible_range() {
        let diags = lint("if (x > 5 && x < 3) {}");
        assert_eq!(diags.len(), 1, "impossible range should be flagged");
    }

    #[test]
    fn test_flags_tautological_or() {
        let diags = lint("if (x > 3 || x < 5) {}");
        assert_eq!(diags.len(), 1, "tautological OR should be flagged");
    }

    #[test]
    fn test_allows_valid_range() {
        let diags = lint("if (x > 3 && x < 5) {}");
        assert!(diags.is_empty(), "valid range should not be flagged");
    }

    #[test]
    fn test_allows_different_variables() {
        let diags = lint("if (x > 5 && y < 3) {}");
        assert!(
            diags.is_empty(),
            "different variables should not be flagged"
        );
    }
}
