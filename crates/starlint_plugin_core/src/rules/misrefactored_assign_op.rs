//! Rule: `misrefactored-assign-op` (OXC)
//!
//! Detect misrefactored assignment operators like `a -= a - b` which was
//! probably meant to be `a -= b` (or `a = a - b`). Similarly `a += a + b`
//! probably meant `a += b`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{AssignmentOperator, BinaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags misrefactored compound assignment operators.
#[derive(Debug)]
pub struct MisrefactoredAssignOp;

impl LintRule for MisrefactoredAssignOp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "misrefactored-assign-op".to_owned(),
            description: "Detect `a -= a - b` (probably meant `a -= b`)".to_owned(),
            category: Category::Correctness,
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

        // Only check compound assignment operators
        let corresponding_binary = match assign.operator {
            AssignmentOperator::Addition => Some(BinaryOperator::Addition),
            AssignmentOperator::Subtraction => Some(BinaryOperator::Subtraction),
            AssignmentOperator::Multiplication => Some(BinaryOperator::Multiplication),
            AssignmentOperator::Division => Some(BinaryOperator::Division),
            AssignmentOperator::Remainder => Some(BinaryOperator::Remainder),
            _ => None,
        };

        let Some(expected_op) = corresponding_binary else {
            return;
        };

        // The RHS should be a binary expression with the same operator
        let Some(AstNode::BinaryExpression(rhs_bin)) = ctx.node(assign.right) else {
            return;
        };

        if rhs_bin.operator != expected_op {
            return;
        }

        // Check if the left side of the binary expression matches the assignment target
        // e.g., `a -= a - b` -> the `a` in `a - b` matches the `a` being assigned
        let target_span = node_id_span(assign.left, ctx);
        let lhs_span = node_id_span(rhs_bin.left, ctx);

        if let (Some(target), Some(lhs)) = (target_span, lhs_span) {
            // Compare by source text length (same identifier = same span length at same content)
            let source = ctx.source_text();
            let target_src = &source[target.0..target.1];
            let lhs_src = &source[lhs.0..lhs.1];

            if target_src == lhs_src {
                let op_str = match assign.operator {
                    AssignmentOperator::Addition => "+=",
                    AssignmentOperator::Subtraction => "-=",
                    AssignmentOperator::Multiplication => "*=",
                    AssignmentOperator::Division => "/=",
                    AssignmentOperator::Remainder => "%=",
                    _ => return,
                };

                // Fix: replace the RHS binary expression with just the right operand
                let rhs_right_span = ctx.node(rhs_bin.right).map(starlint_ast::AstNode::span);
                let (rhs_right_start, rhs_right_end) = rhs_right_span.map_or((0, 0), |s| {
                    (
                        usize::try_from(s.start).unwrap_or(0),
                        usize::try_from(s.end).unwrap_or(0),
                    )
                });
                let right_text = source.get(rhs_right_start..rhs_right_end).unwrap_or("");

                let rhs_bin_span = rhs_bin.span;
                ctx.report(Diagnostic {
                    rule_name: "misrefactored-assign-op".to_owned(),
                    message: format!(
                        "`{target_src} {op_str} {target_src} ...` looks like a misrefactored \
                         compound assignment — did you mean `{target_src} {op_str} ...` without \
                         repeating `{target_src}`?"
                    ),
                    span: Span::new(assign.span.start, assign.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Simplify to `{target_src} {op_str} {right_text}`")),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Simplify to `{target_src} {op_str} {right_text}`"),
                        edits: vec![Edit {
                            span: Span::new(rhs_bin_span.start, rhs_bin_span.end),
                            replacement: right_text.to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

/// Get the span (as usize range) of a node by its ID.
fn node_id_span(id: NodeId, ctx: &LintContext<'_>) -> Option<(usize, usize)> {
    let node = ctx.node(id)?;
    let span = node.span();
    usize::try_from(span.start)
        .ok()
        .zip(usize::try_from(span.end).ok())
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(MisrefactoredAssignOp);

    #[test]
    fn test_flags_subtract_assign() {
        let diags = lint("a -= a - b;");
        assert_eq!(diags.len(), 1, "a -= a - b should be flagged");
    }

    #[test]
    fn test_flags_add_assign() {
        let diags = lint("a += a + b;");
        assert_eq!(diags.len(), 1, "a += a + b should be flagged");
    }

    #[test]
    fn test_allows_correct_assign() {
        let diags = lint("a -= b;");
        assert!(
            diags.is_empty(),
            "correct compound assignment should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_target() {
        let diags = lint("a -= b - c;");
        assert!(
            diags.is_empty(),
            "different target in RHS should not be flagged"
        );
    }

    #[test]
    fn test_allows_simple_assignment() {
        let diags = lint("a = a - b;");
        assert!(diags.is_empty(), "simple assignment should not be flagged");
    }
}
