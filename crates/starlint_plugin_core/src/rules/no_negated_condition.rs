//! Rule: `no-negated-condition`
//!
//! Disallow negated conditions in `if` statements with an `else` branch
//! and in ternary operators. These are harder to read and should be
//! inverted for clarity.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags negated conditions that should be inverted.
#[derive(Debug)]
pub struct NoNegatedCondition;

impl LintRule for NoNegatedCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-negated-condition".to_owned(),
            description: "Disallow negated conditions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ConditionalExpression, AstNodeType::IfStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::IfStatement(stmt) => {
                // Only flag if there is an else branch
                if stmt.alternate.is_none() {
                    return;
                }
                if is_negated(stmt.test, ctx) {
                    // if-else autofix is too complex (multi-line blocks), just report
                    ctx.report_warning(
                        "no-negated-condition",
                        "Unexpected negated condition in `if` with `else`",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstNode::ConditionalExpression(expr) => {
                if is_negated(expr.test, ctx) {
                    let source = ctx.source_text();
                    let negated_text = negate_condition(expr.test, source, ctx);
                    let cons_text = span_text(expr.consequent, source, ctx).to_owned();
                    let alt_text = span_text(expr.alternate, source, ctx).to_owned();

                    // `!x ? a : b` -> `x ? b : a`
                    let replacement = format!("{negated_text} ? {alt_text} : {cons_text}");

                    ctx.report(Diagnostic {
                        rule_name: "no-negated-condition".to_owned(),
                        message: "Unexpected negated condition in ternary".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Invert the condition and swap branches".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Invert condition and swap branches".to_owned(),
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
            _ => {}
        }
    }
}

/// Get source text for a node's span.
fn span_text<'s>(id: NodeId, source: &'s str, ctx: &LintContext<'_>) -> &'s str {
    let sp = ctx.node(id).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );
    let start = usize::try_from(sp.start).unwrap_or(0);
    let end = usize::try_from(sp.end).unwrap_or(start);
    source.get(start..end).unwrap_or_default()
}

/// Produce the negated form of a condition.
/// `!x` -> `x`, `a !== b` -> `a === b`, `a != b` -> `a == b`.
fn negate_condition(id: NodeId, source: &str, ctx: &LintContext<'_>) -> String {
    let Some(node) = ctx.node(id) else {
        return span_text(id, source, ctx).to_owned();
    };
    match node {
        AstNode::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            span_text(unary.argument, source, ctx).to_owned()
        }
        AstNode::BinaryExpression(binary) => {
            let left = span_text(binary.left, source, ctx);
            let right = span_text(binary.right, source, ctx);
            let new_op = match binary.operator {
                BinaryOperator::StrictInequality => "===",
                BinaryOperator::Inequality => "==",
                _ => return span_text(id, source, ctx).to_owned(),
            };
            format!("{left} {new_op} {right}")
        }
        _ => span_text(id, source, ctx).to_owned(),
    }
}

/// Check if an expression is a negation (`!x`) or inequality (`a !== b`, `a != b`).
fn is_negated(id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(node) = ctx.node(id) else {
        return false;
    };
    match node {
        AstNode::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        AstNode::BinaryExpression(binary) => {
            matches!(
                binary.operator,
                BinaryOperator::StrictInequality | BinaryOperator::Inequality
            )
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNegatedCondition);

    #[test]
    fn test_flags_negated_if_with_else() {
        let diags = lint("if (!x) { a(); } else { b(); }");
        assert_eq!(diags.len(), 1, "negated if with else should be flagged");
    }

    #[test]
    fn test_allows_negated_if_without_else() {
        let diags = lint("if (!x) { a(); }");
        assert!(
            diags.is_empty(),
            "negated if without else should not be flagged"
        );
    }

    #[test]
    fn test_flags_negated_ternary() {
        let diags = lint("var r = !x ? a : b;");
        assert_eq!(diags.len(), 1, "negated ternary should be flagged");
    }

    #[test]
    fn test_allows_non_negated_ternary() {
        let diags = lint("var r = x ? a : b;");
        assert!(
            diags.is_empty(),
            "non-negated ternary should not be flagged"
        );
    }

    #[test]
    fn test_flags_inequality_if_with_else() {
        let diags = lint("if (a !== b) { x(); } else { y(); }");
        assert_eq!(diags.len(), 1, "inequality if with else should be flagged");
    }
}
