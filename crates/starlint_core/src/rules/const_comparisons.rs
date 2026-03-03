//! Rule: `const-comparisons` (OXC)
//!
//! Detect comparisons that are always true or always false due to constant
//! operands. For example, `x > 5 && x > 3` (the second check is redundant)
//! or `x > 5 && x < 3` (always false — impossible range).

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, LogicalOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags always-true or always-false constant comparisons.
#[derive(Debug)]
pub struct ConstComparisons;

impl NativeRule for ConstComparisons {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "const-comparisons".to_owned(),
            description: "Detect always-true or always-false constant comparisons".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::LogicalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::LogicalExpression(logical) = kind else {
            return;
        };

        let Expression::BinaryExpression(left) = &logical.left else {
            return;
        };
        let Expression::BinaryExpression(right) = &logical.right else {
            return;
        };

        // Both sides must have one variable operand and one numeric literal
        let source = ctx.source_text();
        let left_info = extract_comparison_info(left, source);
        let right_info = extract_comparison_info(right, source);

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
            ctx.report_warning(
                "const-comparisons",
                message,
                Span::new(logical.span.start, logical.span.end),
            );
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
    expr: &oxc_ast::ast::BinaryExpression<'_>,
    source: &'s str,
) -> Option<ComparisonInfo<'s>> {
    if !expr.operator.is_compare() {
        return None;
    }

    let left_num = get_numeric_value(&expr.left);
    let right_num = get_numeric_value(&expr.right);

    match (left_num, right_num) {
        // variable OP number
        (None, Some(value)) => {
            let start = usize::try_from(expr.left.span().start).ok()?;
            let end = usize::try_from(expr.left.span().end).ok()?;
            let var_text = source.get(start..end)?;
            Some(ComparisonInfo {
                var_text,
                op: expr.operator,
                value,
            })
        }
        // number OP variable → flip the operator
        (Some(value), None) => {
            let start = usize::try_from(expr.right.span().start).ok()?;
            let end = usize::try_from(expr.right.span().end).ok()?;
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

use oxc_span::GetSpan;

/// Get the numeric value from an expression if it's a numeric literal.
fn get_numeric_value(expr: &Expression<'_>) -> Option<f64> {
    match expr {
        Expression::NumericLiteral(n) => Some(n.value),
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
/// e.g., x > 5 && x < 3 → impossible.
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
/// e.g., x > 3 || x < 5 → always true (covers everything).
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConstComparisons)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

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
