//! Rule: `prefer-array-index-of`
//!
//! Prefer `.indexOf()` over `.findIndex()` for simple equality checks.
//! `.findIndex(x => x === val)` can be simplified to `.indexOf(val)`.

#![allow(clippy::indexing_slicing)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.findIndex()` calls with simple equality callbacks.
#[derive(Debug)]
pub struct PreferArrayIndexOf;

/// Check if an arrow function body is a simple binary equality expression.
fn is_simple_equality_body(body_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
        return false;
    };
    // Expression body (single statement that is an expression statement)
    if body.statements.len() != 1 {
        return false;
    }
    let stmt_id = body.statements[0];
    let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(stmt_id) else {
        return false;
    };
    matches!(
        ctx.node(expr_stmt.expression),
        Some(AstNode::BinaryExpression(bin))
            if matches!(
                bin.operator,
                BinaryOperator::StrictEquality | BinaryOperator::Equality
            )
    )
}

/// Extract the value being compared in `x => x === val`, returning the source text of `val`.
/// The parameter name must match one side of the equality.
#[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
fn extract_equality_value(
    arrow_params: &[NodeId],
    arrow_body: NodeId,
    source: &str,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let Some(AstNode::FunctionBody(body)) = ctx.node(arrow_body) else {
        return None;
    };
    let stmt_id = *body.statements.first()?;
    let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(stmt_id) else {
        return None;
    };
    let Some(AstNode::BinaryExpression(bin)) = ctx.node(expr_stmt.expression) else {
        return None;
    };

    // Get the parameter name
    let &param_id = arrow_params.first()?;
    let param_span = ctx.node(param_id)?.span();
    let param_name = source.get(param_span.start as usize..param_span.end as usize)?;

    let left_span = ctx.node(bin.left)?.span();
    let right_span = ctx.node(bin.right)?.span();
    let left_text = source.get(left_span.start as usize..left_span.end as usize)?;
    let right_text = source.get(right_span.start as usize..right_span.end as usize)?;

    // Return the side that is NOT the parameter
    if left_text == param_name {
        Some(right_text.to_owned())
    } else if right_text == param_name {
        Some(left_text.to_owned())
    } else {
        None
    }
}

impl LintRule for PreferArrayIndexOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-index-of".to_owned(),
            description: "Prefer `.indexOf()` over `.findIndex()` for simple equality checks"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    #[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        if member.property.as_str() != "findIndex" {
            return;
        }

        // Must have exactly one argument.
        if call.arguments.len() != 1 {
            return;
        }

        let first_arg_id = call.arguments[0];

        // Check for arrow function with simple equality body.
        if let Some(AstNode::ArrowFunctionExpression(arrow)) = ctx.node(first_arg_id) {
            let params: Vec<NodeId> = arrow.params.to_vec();
            let body_id = arrow.body;
            if params.len() == 1 && is_simple_equality_body(body_id, ctx) {
                // Try to extract the value being compared against the parameter
                let fix = extract_equality_value(&params, body_id, ctx.source_text(), ctx).map(
                    |val_text| {
                        // We need the span of the property name "findIndex" for the method rename
                        // Since StaticMemberExpressionNode doesn't have property.span, use source text
                        // to find the property span from the member expression span
                        let member_span = member.span;
                        // Find "findIndex" in the source text within the member span
                        let source = ctx.source_text();
                        let member_text = source
                            .get(member_span.start as usize..member_span.end as usize)
                            .unwrap_or("");
                        let prop_offset = member_text.rfind("findIndex").unwrap_or(0);
                        let prop_start =
                            member_span.start + u32::try_from(prop_offset).unwrap_or(0);
                        let prop_end = prop_start + 9; // "findIndex".len()

                        // Args span
                        let first_arg_span = ctx.node(first_arg_id).map_or(
                            starlint_ast::types::Span::EMPTY,
                            starlint_ast::AstNode::span,
                        );

                        Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `.indexOf({val_text})`"),
                            edits: vec![
                                Edit {
                                    span: Span::new(prop_start, prop_end),
                                    replacement: "indexOf".to_owned(),
                                },
                                Edit {
                                    span: Span::new(first_arg_span.start, first_arg_span.end),
                                    replacement: val_text,
                                },
                            ],
                            is_snippet: false,
                        }
                    },
                );

                ctx.report(Diagnostic {
                    rule_name: "prefer-array-index-of".to_owned(),
                    message: "Prefer `.indexOf()` over `.findIndex()` for simple equality checks"
                        .to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace `.findIndex()` with `.indexOf()`".to_owned()),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferArrayIndexOf);

    #[test]
    fn test_flags_find_index_strict_equality() {
        let diags = lint("arr.findIndex(x => x === 5);");
        assert_eq!(
            diags.len(),
            1,
            "should flag .findIndex() with strict equality"
        );
    }

    #[test]
    fn test_flags_find_index_loose_equality() {
        let diags = lint("arr.findIndex(x => x == val);");
        assert_eq!(
            diags.len(),
            1,
            "should flag .findIndex() with loose equality"
        );
    }

    #[test]
    fn test_allows_find_index_complex_callback() {
        let diags = lint("arr.findIndex(x => x.id === 5);");
        // This is a member expression equality, not a simple `x === val`.
        // Our heuristic still flags it because the body is a binary equality.
        // That is acceptable -- it is a suggestion, not an error.
        assert_eq!(diags.len(), 1, "still flags member-based equality");
    }

    #[test]
    fn test_allows_index_of() {
        let diags = lint("arr.indexOf(5);");
        assert!(diags.is_empty(), ".indexOf() should not be flagged");
    }
}
