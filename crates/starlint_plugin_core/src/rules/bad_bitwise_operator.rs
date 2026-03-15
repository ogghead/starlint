//! Rule: `bad-bitwise-operator` (OXC)
//!
//! Catch likely typos where `|` was used instead of `||` or `&` instead of
//! `&&`. This flags bitwise operators used with boolean operands (comparisons
//! or boolean literals).

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags bitwise `|` / `&` when both operands look boolean.
#[derive(Debug)]
pub struct BadBitwiseOperator;

impl LintRule for BadBitwiseOperator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-bitwise-operator".to_owned(),
            description: "Catch `|` vs `||` and `&` vs `&&` operator typos".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
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

        let intended = match expr.operator {
            BinaryOperator::BitwiseOR => "||",
            BinaryOperator::BitwiseAnd => "&&",
            _ => return,
        };

        let left_bool = ctx.node(expr.left).is_some_and(|n| looks_boolean(ctx, n));
        let right_bool = ctx.node(expr.right).is_some_and(|n| looks_boolean(ctx, n));

        if !left_bool || !right_bool {
            return;
        }

        let actual = if intended == "||" { "|" } else { "&" };

        let source = ctx.source_text();
        let left_end = ctx.node(expr.left).map_or(0, |n| n.span().end as usize);
        let right_start = ctx.node(expr.right).map_or(0, |n| n.span().start as usize);
        let between = source.get(left_end..right_start).unwrap_or("");

        let fix = between.find(actual).map(|offset| {
            let op_pos = u32::try_from(left_end.saturating_add(offset)).unwrap_or(0);
            Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `{actual}` with `{intended}`"),
                edits: vec![Edit {
                    span: Span::new(op_pos, op_pos.saturating_add(1)),
                    replacement: intended.to_owned(),
                }],
                is_snippet: false,
            }
        });

        ctx.report(Diagnostic {
            rule_name: "bad-bitwise-operator".to_owned(),
            message: format!("Suspicious use of `{actual}` — did you mean `{intended}`?"),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{actual}` with `{intended}`")),
            fix,
            labels: vec![],
        });
    }
}

/// Heuristic: does this node look like it produces a boolean?
fn looks_boolean(_ctx: &LintContext<'_>, node: &AstNode) -> bool {
    match node {
        // Boolean literals and logical expressions always produce booleans
        AstNode::BooleanLiteral(_) | AstNode::LogicalExpression(_) => true,
        // Comparisons produce booleans
        AstNode::BinaryExpression(bin) => bin.operator.is_equality() || bin.operator.is_compare(),
        // !x produces a boolean
        AstNode::UnaryExpression(un) => un.operator == UnaryOperator::LogicalNot,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(BadBitwiseOperator);

    #[test]
    fn test_flags_pipe_with_booleans() {
        let diags = lint("if (a > 1 | b > 2) {}");
        assert_eq!(
            diags.len(),
            1,
            "bitwise OR with boolean operands should be flagged"
        );
    }

    #[test]
    fn test_flags_ampersand_with_booleans() {
        let diags = lint("if (a === 1 & b === 2) {}");
        assert_eq!(
            diags.len(),
            1,
            "bitwise AND with boolean operands should be flagged"
        );
    }

    #[test]
    fn test_allows_bitwise_with_numbers() {
        let diags = lint("var n = a | b;");
        assert!(
            diags.is_empty(),
            "bitwise OR with non-boolean operands should not be flagged"
        );
    }

    #[test]
    fn test_allows_logical_or() {
        let diags = lint("if (a > 1 || b > 2) {}");
        assert!(diags.is_empty(), "logical OR should not be flagged");
    }

    #[test]
    fn test_flags_boolean_literal_pipe() {
        let diags = lint("var x = true | false;");
        assert_eq!(
            diags.len(),
            1,
            "bitwise OR with boolean literals should be flagged"
        );
    }
}
