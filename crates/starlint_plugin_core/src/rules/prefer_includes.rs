//! Rule: `prefer-includes`
//!
//! Prefer `.includes()` over `.indexOf()` existence checks.
//! `arr.indexOf(x) !== -1` should be `arr.includes(x)`.
//! `arr.indexOf(x) === -1` should be `!arr.includes(x)`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.indexOf()` existence checks that can use `.includes()`.
#[derive(Debug)]
pub struct PreferIncludes;

/// Check if an `AstNode` is the numeric literal `-1`.
///
/// In the flat AST, `-1` is `UnaryExpression(UnaryNegation, NumericLiteral(1.0))`.
fn is_negative_one(ctx: &LintContext<'_>, node: &AstNode) -> bool {
    if let AstNode::UnaryExpression(unary) = node {
        if unary.operator == UnaryOperator::UnaryNegation {
            if let Some(AstNode::NumericLiteral(lit)) = ctx.node(unary.argument) {
                return (lit.value - 1.0).abs() < f64::EPSILON;
            }
        }
    }
    false
}

/// Check if an `AstNode` is the numeric literal `0`.
fn is_zero(node: &AstNode) -> bool {
    if let AstNode::NumericLiteral(lit) = node {
        return lit.value.abs() < f64::EPSILON;
    }
    false
}

/// Determine whether the comparison means "includes" (true) or "not includes" (false).
///
/// Returns `None` if the operator/comparand combination doesn't map to an includes check.
const fn classify_includes(
    operator: BinaryOperator,
    index_of_on_left: bool,
    comparand_is_negative_one: bool,
) -> Option<bool> {
    match (index_of_on_left, operator, comparand_is_negative_one) {
        (_, BinaryOperator::StrictInequality | BinaryOperator::Inequality, true)
        | (true, BinaryOperator::GreaterThan, true)
        | (true, BinaryOperator::GreaterEqualThan, false)
        | (false, BinaryOperator::LessThan, true)
        | (false, BinaryOperator::LessEqualThan, false) => Some(true),

        (_, BinaryOperator::StrictEquality | BinaryOperator::Equality, true)
        | (true, BinaryOperator::LessThan, false)
        | (false, BinaryOperator::GreaterThan, false) => Some(false),

        _ => None,
    }
}

impl LintRule for PreferIncludes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-includes".to_owned(),
            description: "Prefer `.includes()` over `.indexOf()` existence checks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        let left_node = ctx.node(expr.left);
        let right_node = ctx.node(expr.right);
        let (Some(left), Some(right)) = (left_node, right_node) else {
            return;
        };

        // Try to find an indexOf call on either side of the binary expression.
        let (index_of_on_left, call, comparand) = match (left, right) {
            (AstNode::CallExpression(call), comparand) => (true, call, comparand),
            (comparand, AstNode::CallExpression(call)) => (false, call, comparand),
            _ => return,
        };

        // Callee must be a .indexOf() member call.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        if member.property.as_str() != "indexOf" {
            return;
        }

        // Only match when indexOf has exactly 1 argument (skip fromIndex overloads).
        if call.arguments.len() != 1 {
            return;
        }

        // Comparand must be -1 or 0.
        let comparand_is_neg_one = is_negative_one(ctx, comparand);
        let comparand_is_zero = is_zero(comparand);
        if !comparand_is_neg_one && !comparand_is_zero {
            return;
        }

        let Some(positive) =
            classify_includes(expr.operator, index_of_on_left, comparand_is_neg_one)
        else {
            return;
        };

        // Build the replacement: `obj.includes(arg)` or `!obj.includes(arg)`
        let obj_span = ctx.node(member.object).map_or(
            starlint_ast::types::Span::new(0, 0),
            starlint_ast::AstNode::span,
        );
        let obj_start = usize::try_from(obj_span.start).unwrap_or(0);
        let obj_end = usize::try_from(obj_span.end).unwrap_or(0);
        let Some(obj_text) = ctx.source_text().get(obj_start..obj_end) else {
            return;
        };

        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };
        let arg_span = ctx.node(first_arg_id).map_or(
            starlint_ast::types::Span::new(0, 0),
            starlint_ast::AstNode::span,
        );
        let arg_start = usize::try_from(arg_span.start).unwrap_or(0);
        let arg_end = usize::try_from(arg_span.end).unwrap_or(0);
        let Some(arg_text) = ctx.source_text().get(arg_start..arg_end) else {
            return;
        };

        let replacement = if positive {
            format!("{obj_text}.includes({arg_text})")
        } else {
            format!("!{obj_text}.includes({arg_text})")
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-includes".to_owned(),
            message: "Use `.includes()` instead of `.indexOf()` existence check".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace with `{replacement}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
                    replacement,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferIncludes);

    #[test]
    fn test_flags_not_equals_negative_one() {
        let diags = lint("if (arr.indexOf(x) !== -1) {}");
        assert_eq!(diags.len(), 1, "should flag !== -1");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("arr.includes(x)"),
            "fix should use .includes()"
        );
    }

    #[test]
    fn test_flags_gte_zero() {
        let diags = lint("if (arr.indexOf(x) >= 0) {}");
        assert_eq!(diags.len(), 1, "should flag >= 0");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("arr.includes(x)"),
        );
    }

    #[test]
    fn test_flags_gt_negative_one() {
        let diags = lint("if (arr.indexOf(x) > -1) {}");
        assert_eq!(diags.len(), 1, "should flag > -1");
    }

    #[test]
    fn test_flags_equals_negative_one_negated() {
        let diags = lint("if (arr.indexOf(x) === -1) {}");
        assert_eq!(diags.len(), 1, "should flag === -1");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("!arr.includes(x)"),
            "fix should use !.includes()"
        );
    }

    #[test]
    fn test_flags_lt_zero_negated() {
        let diags = lint("if (arr.indexOf(x) < 0) {}");
        assert_eq!(diags.len(), 1, "should flag < 0");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("!arr.includes(x)"),
        );
    }

    #[test]
    fn test_flags_reversed_order() {
        let diags = lint("if (-1 !== arr.indexOf(x)) {}");
        assert_eq!(diags.len(), 1, "should flag reversed order");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("arr.includes(x)"),
        );
    }

    #[test]
    fn test_flags_string_indexof() {
        let diags = lint("if (str.indexOf('a') > -1) {}");
        assert_eq!(diags.len(), 1, "should flag string .indexOf()");
    }

    #[test]
    fn test_allows_index_of_alone() {
        let diags = lint("const i = arr.indexOf(x);");
        assert!(
            diags.is_empty(),
            "indexOf without comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_position_check() {
        let diags = lint("if (arr.indexOf(x) > 0) {}");
        assert!(diags.is_empty(), "> 0 is a position check, not existence");
    }

    #[test]
    fn test_allows_from_index() {
        let diags = lint("if (arr.indexOf(x, 5) !== -1) {}");
        assert!(
            diags.is_empty(),
            "indexOf with fromIndex should not be flagged"
        );
    }

    #[test]
    fn test_allows_last_index_of() {
        let diags = lint("if (arr.lastIndexOf(x) !== -1) {}");
        assert!(diags.is_empty(), "lastIndexOf should not be flagged");
    }
}
