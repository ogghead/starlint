//! Rule: `yoda`
//!
//! Disallow "Yoda conditions" where the literal comes before the variable
//! in a comparison, e.g. `"red" === color` instead of `color === "red"`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags Yoda conditions (literal on the left of a comparison).
#[derive(Debug)]
pub struct Yoda;

impl LintRule for Yoda {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "yoda".to_owned(),
            description: "Disallow Yoda conditions".to_owned(),
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

        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        // Flag: literal on the left, non-literal on the right
        if is_literal(expr.left, ctx) && !is_literal(expr.right, ctx) {
            let source = ctx.source_text();
            let (Some(left_node), Some(right_node)) = (ctx.node(expr.left), ctx.node(expr.right))
            else {
                return;
            };
            let left_span = left_node.span();
            let right_span = right_node.span();
            let left_start = usize::try_from(left_span.start).unwrap_or(0);
            let left_end = usize::try_from(left_span.end).unwrap_or(0);
            let right_start = usize::try_from(right_span.start).unwrap_or(0);
            let right_end = usize::try_from(right_span.end).unwrap_or(0);
            let left_text = source.get(left_start..left_end).unwrap_or("");
            let right_text = source.get(right_start..right_end).unwrap_or("");
            let flipped = flip_comparison(expr.operator);

            ctx.report(Diagnostic {
                rule_name: "yoda".to_owned(),
                message: "Expected literal to be on the right side of comparison".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some("Swap the operands".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Swap operands".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: format!("{right_text} {flipped} {left_text}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Flip a comparison operator for swapping operands.
const fn flip_comparison(op: BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::LessThan => ">",
        BinaryOperator::LessEqualThan => ">=",
        BinaryOperator::GreaterThan => "<",
        BinaryOperator::GreaterEqualThan => "<=",
        BinaryOperator::Equality => "==",
        BinaryOperator::Inequality => "!=",
        BinaryOperator::StrictInequality => "!==",
        _ => "===",
    }
}

/// Check if a node (by ID) is a literal value.
fn is_literal(id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.node(id),
        Some(
            AstNode::StringLiteral(_)
                | AstNode::NumericLiteral(_)
                | AstNode::BooleanLiteral(_)
                | AstNode::NullLiteral(_)
        )
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(Yoda);

    #[test]
    fn test_flags_yoda_condition() {
        let diags = lint("if ('red' === color) {}");
        assert_eq!(diags.len(), 1, "Yoda condition should be flagged");
    }

    #[test]
    fn test_allows_normal_condition() {
        let diags = lint("if (color === 'red') {}");
        assert!(diags.is_empty(), "normal condition should not be flagged");
    }

    #[test]
    fn test_allows_two_variables() {
        let diags = lint("if (a === b) {}");
        assert!(diags.is_empty(), "two variables should not be flagged");
    }

    #[test]
    fn test_flags_number_yoda() {
        let diags = lint("if (5 < x) {}");
        assert_eq!(diags.len(), 1, "number yoda condition should be flagged");
    }
}
