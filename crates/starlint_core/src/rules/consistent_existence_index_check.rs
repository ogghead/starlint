//! Rule: `consistent-existence-index-check`
//!
//! Enforce consistent style for checking if an index exists. Prefer
//! `!== -1` over `>= 0` and `=== -1` over `< 0` when checking the
//! result of `indexOf` or `findIndex`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;

/// Method names that return an index (`-1` means not found).
const INDEX_METHODS: &[&str] = &["indexOf", "findIndex"];

/// Flags inconsistent index-existence comparisons.
#[derive(Debug)]
pub struct ConsistentExistenceIndexCheck;

impl LintRule for ConsistentExistenceIndexCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-existence-index-check".to_owned(),
            description: "Enforce consistent style for checking if an index exists".to_owned(),
            category: Category::Style,
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

        // Check pattern: `someCall(...) OP value`
        // where someCall is .indexOf() or .findIndex()
        if !is_index_call(expr.left, ctx) {
            return;
        }

        // (message, fix_description, replacement_operator, replacement_value)
        let fix_info = match expr.operator {
            // `indexOf(x) >= 0` → prefer `indexOf(x) !== -1`
            BinaryOperator::GreaterEqualThan if is_numeric_literal(expr.right, 0.0, ctx) => Some((
                "Use `!== -1` instead of `>= 0` for index existence check",
                "Replace `>= 0` with `!== -1`",
                "!==",
                "-1",
            )),
            // `indexOf(x) > -1` → prefer `indexOf(x) !== -1`
            BinaryOperator::GreaterThan if is_numeric_literal(expr.right, -1.0, ctx) => Some((
                "Use `!== -1` instead of `> -1` for index existence check",
                "Replace `> -1` with `!== -1`",
                "!==",
                "-1",
            )),
            // `indexOf(x) < 0` → prefer `indexOf(x) === -1`
            BinaryOperator::LessThan if is_numeric_literal(expr.right, 0.0, ctx) => Some((
                "Use `=== -1` instead of `< 0` for index non-existence check",
                "Replace `< 0` with `=== -1`",
                "===",
                "-1",
            )),
            _ => None,
        };

        if let Some((message, fix_desc, new_op, new_val)) = fix_info {
            // Find the operator span between left and right expressions
            let left_end = ctx.node(expr.left).map_or(0, |n| n.span().end);
            let right_end = ctx.node(expr.right).map_or(0, |n| n.span().end);

            ctx.report(Diagnostic {
                rule_name: "consistent-existence-index-check".to_owned(),
                message: message.to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some(fix_desc.to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: fix_desc.to_owned(),
                    edits: vec![Edit {
                        span: Span::new(left_end, right_end),
                        replacement: format!(" {new_op} {new_val}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a call to `.indexOf()` or `.findIndex()`.
fn is_index_call(id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::CallExpression(call)) = ctx.node(id) else {
        return false;
    };

    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return false;
    };

    let method_name = member.property.as_str();
    INDEX_METHODS.contains(&method_name)
}

/// Check if an expression is a numeric literal with a specific value.
fn is_numeric_literal(id: NodeId, value: f64, ctx: &LintContext<'_>) -> bool {
    let Some(node) = ctx.node(id) else {
        return false;
    };

    // Handle negative numbers: `-1` is parsed as `UnaryExpression(-, 1)`
    if let AstNode::UnaryExpression(unary) = node {
        if unary.operator == UnaryOperator::UnaryNegation {
            if let Some(AstNode::NumericLiteral(lit)) = ctx.node(unary.argument) {
                return ((-lit.value) - value).abs() < f64::EPSILON;
            }
        }
        return false;
    }

    let AstNode::NumericLiteral(lit) = node else {
        return false;
    };
    (lit.value - value).abs() < f64::EPSILON
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentExistenceIndexCheck)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_gte_zero() {
        let diags = lint("if (arr.indexOf(x) >= 0) {}");
        assert_eq!(diags.len(), 1, "`>= 0` index check should be flagged");
    }

    #[test]
    fn test_allows_not_equals_neg_one() {
        let diags = lint("if (arr.indexOf(x) !== -1) {}");
        assert!(
            diags.is_empty(),
            "`!== -1` index check should not be flagged"
        );
    }

    #[test]
    fn test_flags_gt_neg_one() {
        let diags = lint("if (arr.findIndex(x => x > 0) > -1) {}");
        assert_eq!(diags.len(), 1, "`> -1` index check should be flagged");
    }

    #[test]
    fn test_allows_equals_neg_one() {
        let diags = lint("if (str.indexOf('a') === -1) {}");
        assert!(
            diags.is_empty(),
            "`=== -1` index check should not be flagged"
        );
    }

    #[test]
    fn test_flags_lt_zero() {
        let diags = lint("if (arr.indexOf(x) < 0) {}");
        assert_eq!(diags.len(), 1, "`< 0` index check should be flagged");
    }

    #[test]
    fn test_allows_unrelated_comparison() {
        let diags = lint("if (arr.length >= 0) {}");
        assert!(
            diags.is_empty(),
            "non-indexOf comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_equals_zero() {
        let diags = lint("if (arr.indexOf(x) === 0) {}");
        assert!(diags.is_empty(), "`=== 0` is a valid specific-index check");
    }
}
