//! Rule: `double-comparisons` (OXC)
//!
//! Detect `a >= b && a <= b` which can be simplified to `a === b`, and
//! `a > b || a < b` which can be simplified to `a !== b`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, LogicalOperator};
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags redundant double comparisons that can be simplified.
#[derive(Debug)]
pub struct DoubleComparisons;

impl LintRule for DoubleComparisons {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "double-comparisons".to_owned(),
            description: "Detect `a >= b && a <= b` (simplify to `a === b`)".to_owned(),
            category: Category::Style,
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

        // Extract source text for operand comparison
        let source = ctx.source_text();
        let same_operands = (src_equal(ctx, left.left, right.left, source)
            && src_equal(ctx, left.right, right.right, source))
            || (src_equal(ctx, left.left, right.right, source)
                && src_equal(ctx, left.right, right.left, source));

        if !same_operands {
            return;
        }

        let finding = match logical.operator {
            LogicalOperator::And => {
                let pair = (left.operator, right.operator);
                matches!(
                    pair,
                    (
                        BinaryOperator::GreaterEqualThan,
                        BinaryOperator::LessEqualThan
                    ) | (
                        BinaryOperator::LessEqualThan,
                        BinaryOperator::GreaterEqualThan
                    )
                )
                .then_some("This double comparison can be simplified to `===`")
            }
            LogicalOperator::Or => {
                let pair = (left.operator, right.operator);
                matches!(
                    pair,
                    (BinaryOperator::GreaterThan, BinaryOperator::LessThan)
                        | (BinaryOperator::LessThan, BinaryOperator::GreaterThan)
                )
                .then_some("This double comparison can be simplified to `!==`")
            }
            LogicalOperator::Coalesce => None,
        };

        if let Some(message) = finding {
            let replacement_op = match logical.operator {
                LogicalOperator::And => "===",
                _ => "!==",
            };
            let left_a = node_source(ctx, left.left, source).unwrap_or("");
            let right_b = node_source(ctx, left.right, source).unwrap_or("");

            ctx.report(Diagnostic {
                rule_name: "double-comparisons".to_owned(),
                message: message.to_owned(),
                span: Span::new(logical.span.start, logical.span.end),
                severity: Severity::Warning,
                help: Some(format!("Simplify to `{replacement_op}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Simplify to `{replacement_op}`"),
                    edits: vec![Edit {
                        span: Span::new(logical.span.start, logical.span.end),
                        replacement: format!("{left_a} {replacement_op} {right_b}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Compare two nodes by their source text content.
fn src_equal(ctx: &LintContext<'_>, a: NodeId, b: NodeId, source: &str) -> bool {
    let a_slice = node_source(ctx, a, source);
    let b_slice = node_source(ctx, b, source);
    match (a_slice, b_slice) {
        (Some(a_str), Some(b_str)) => a_str == b_str,
        _ => false,
    }
}

/// Extract the source text slice for a node.
#[allow(clippy::as_conversions)]
fn node_source<'s>(ctx: &LintContext<'_>, id: NodeId, source: &'s str) -> Option<&'s str> {
    let node = ctx.node(id)?;
    let span = node.span();
    source.get(span.start as usize..span.end as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(DoubleComparisons)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_gte_and_lte() {
        let diags = lint("if (a >= b && a <= b) {}");
        assert_eq!(
            diags.len(),
            1,
            "a >= b && a <= b should be flagged (simplify to ===)"
        );
    }

    #[test]
    fn test_flags_gt_or_lt() {
        let diags = lint("if (a > b || a < b) {}");
        assert_eq!(
            diags.len(),
            1,
            "a > b || a < b should be flagged (simplify to !==)"
        );
    }

    #[test]
    fn test_allows_different_operands() {
        let diags = lint("if (a >= b && c <= d) {}");
        assert!(diags.is_empty(), "different operands should not be flagged");
    }

    #[test]
    fn test_allows_normal_range_check() {
        let diags = lint("if (a >= 0 && a <= 10) {}");
        assert!(
            diags.is_empty(),
            "range check with different bounds should not be flagged"
        );
    }
}
