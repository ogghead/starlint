//! Rule: `operator-assignment`
//!
//! Require or disallow assignment operator shorthand where possible.
//! `x = x + 1` should be written as `x += 1`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{AssignmentOperator, BinaryOperator};
use starlint_ast::types::NodeId;

/// Flags assignments that could use shorthand operators.
#[derive(Debug)]
pub struct OperatorAssignment;

impl LintRule for OperatorAssignment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "operator-assignment".to_owned(),
            description: "Require assignment operator shorthand where possible".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Only check plain assignment (=)
        if assign.operator != AssignmentOperator::Assign {
            return;
        }

        // Right side must be a binary expression
        let Some(AstNode::BinaryExpression(binary)) = ctx.node(assign.right) else {
            return;
        };

        // Check if the binary operator has a compound assignment form
        if !has_shorthand(binary.operator) {
            return;
        }

        // Check if the left side of the binary matches the assignment target
        let source = ctx.source_text();
        if targets_match(assign.left, binary.left, source, ctx) {
            let target_span = ctx.node(assign.left).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let target_start = usize::try_from(target_span.start).unwrap_or(0);
            let target_end = usize::try_from(target_span.end).unwrap_or(0);
            let target_text = source.get(target_start..target_end).unwrap_or("");
            let right_span = ctx.node(binary.right).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let right_start = usize::try_from(right_span.start).unwrap_or(0);
            let right_end = usize::try_from(right_span.end).unwrap_or(0);
            let right_text = source.get(right_start..right_end).unwrap_or("");
            let op_str = shorthand_op_str(binary.operator);

            ctx.report(Diagnostic {
                rule_name: "operator-assignment".to_owned(),
                message: "Assignment can be replaced with an operator assignment".to_owned(),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Warning,
                help: Some(format!("Use `{op_str}` shorthand")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Use `{op_str}` shorthand"),
                    edits: vec![Edit {
                        span: Span::new(assign.span.start, assign.span.end),
                        replacement: format!("{target_text} {op_str} {right_text}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if a binary operator has a corresponding compound assignment operator.
const fn has_shorthand(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
    )
}

/// Convert a binary operator to its compound assignment shorthand string.
const fn shorthand_op_str(op: BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::Addition => "+=",
        BinaryOperator::Subtraction => "-=",
        BinaryOperator::Multiplication => "*=",
        BinaryOperator::Division => "/=",
        BinaryOperator::Remainder => "%=",
        BinaryOperator::Exponential => "**=",
        BinaryOperator::BitwiseAnd => "&=",
        BinaryOperator::BitwiseOR => "|=",
        BinaryOperator::BitwiseXOR => "^=",
        BinaryOperator::ShiftLeft => "<<=",
        BinaryOperator::ShiftRight => ">>=",
        BinaryOperator::ShiftRightZeroFill => ">>>=",
        _ => "=",
    }
}

/// Compare assignment target and expression by source text.
fn targets_match(target_id: NodeId, expr_id: NodeId, source: &str, ctx: &LintContext<'_>) -> bool {
    let target_span = ctx.node(target_id).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );
    let expr_span = ctx.node(expr_id).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );

    let target_start = usize::try_from(target_span.start).unwrap_or(0);
    let target_end = usize::try_from(target_span.end).unwrap_or(0);
    let expr_start = usize::try_from(expr_span.start).unwrap_or(0);
    let expr_end = usize::try_from(expr_span.end).unwrap_or(0);

    let target_text = source.get(target_start..target_end);
    let expr_text = source.get(expr_start..expr_end);

    match (target_text, expr_text) {
        (Some(t), Some(e)) => t == e,
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(OperatorAssignment)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_add_assignment() {
        let diags = lint("x = x + 1;");
        assert_eq!(diags.len(), 1, "x = x + 1 should be flagged");
    }

    #[test]
    fn test_allows_shorthand() {
        let diags = lint("x += 1;");
        assert!(diags.is_empty(), "x += 1 should not be flagged");
    }

    #[test]
    fn test_allows_different_variables() {
        let diags = lint("x = y + 1;");
        assert!(diags.is_empty(), "x = y + 1 should not be flagged");
    }

    #[test]
    fn test_flags_multiply_assignment() {
        let diags = lint("x = x * 2;");
        assert_eq!(diags.len(), 1, "x = x * 2 should be flagged");
    }
}
