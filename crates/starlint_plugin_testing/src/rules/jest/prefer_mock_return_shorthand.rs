//! Rule: `jest/prefer-mock-return-shorthand`
//!
//! Suggest `jest.fn().mockReturnValue(x)` over
//! `jest.fn().mockImplementation(() => x)`. The shorthand is more readable
//! when the mock simply returns a static value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.mockImplementation(() => x)` that could use `.mockReturnValue(x)`.
#[derive(Debug)]
pub struct PreferMockReturnShorthand;

impl LintRule for PreferMockReturnShorthand {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-mock-return-shorthand".to_owned(),
            description:
                "Suggest using `mockReturnValue()` instead of `mockImplementation(() => x)`"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("mockImplementation") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `.mockImplementation(...)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        if member.property.as_str() != "mockImplementation" {
            return;
        }

        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };
        let Some(arg_node) = ctx.node(*first_arg_id) else {
            return;
        };

        // Check for arrow function with expression body and no parameters:
        // `() => x` (but NOT `() => Promise.resolve(x)` which is handled by
        // prefer-mock-promise-shorthand)
        let is_simple_return = match arg_node {
            AstNode::ArrowFunctionExpression(arrow) => {
                // Must be expression body with no parameters
                arrow.expression
                    && arrow.params.is_empty()
                    && !is_promise_call_in_body(ctx, arrow.body)
            }
            _ => false,
        };

        if is_simple_return {
            // Fix: .mockImplementation(() => x) -> .mockReturnValue(x)
            // Extract the return value from the arrow body
            let fix = if let AstNode::ArrowFunctionExpression(arrow) = arg_node {
                extract_arrow_return_fix(ctx, arrow.body, member.object, call.span)
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: "jest/prefer-mock-return-shorthand".to_owned(),
                message: "Use `.mockReturnValue(x)` instead of `.mockImplementation(() => x)`"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Extract the fix for replacing mockImplementation with mockReturnValue.
#[allow(clippy::as_conversions)]
fn extract_arrow_return_fix(
    ctx: &LintContext<'_>,
    body_id: NodeId,
    obj_id: NodeId,
    call_span: starlint_ast::types::Span,
) -> Option<Fix> {
    let AstNode::FunctionBody(body) = ctx.node(body_id)? else {
        return None;
    };
    let first_stmt_id = body.statements.first()?;
    let AstNode::ExpressionStatement(es) = ctx.node(*first_stmt_id)? else {
        return None;
    };
    let val_node = ctx.node(es.expression)?;
    let val_span = val_node.span();
    let source = ctx.source_text();
    let val_text = source.get(val_span.start as usize..val_span.end as usize)?;
    let obj_node = ctx.node(obj_id)?;
    let obj_span = obj_node.span();
    let obj_text = source.get(obj_span.start as usize..obj_span.end as usize)?;
    let replacement = format!("{obj_text}.mockReturnValue({val_text})");
    Some(Fix {
        kind: FixKind::SafeFix,
        message: format!("Replace with `{replacement}`"),
        edits: vec![Edit {
            span: Span::new(call_span.start, call_span.end),
            replacement,
        }],
        is_snippet: false,
    })
}

/// Check if the arrow body contains a `Promise.resolve` or `Promise.reject` call.
fn is_promise_call_in_body(ctx: &LintContext<'_>, body_id: NodeId) -> bool {
    let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
        return false;
    };
    body.statements.first().is_some_and(|stmt_id| {
        let Some(AstNode::ExpressionStatement(es)) = ctx.node(*stmt_id) else {
            return false;
        };
        is_promise_call(ctx, es.expression)
    })
}

/// Check if a node (by ID) is `Promise.resolve(...)` or `Promise.reject(...)`.
fn is_promise_call(ctx: &LintContext<'_>, id: NodeId) -> bool {
    let Some(AstNode::CallExpression(call)) = ctx.node(id) else {
        return false;
    };
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return false;
    };
    let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) else {
        return false;
    };
    if obj.name.as_str() != "Promise" {
        return false;
    }
    let method = member.property.as_str();
    method == "resolve" || method == "reject"
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferMockReturnShorthand);

    #[test]
    fn test_flags_simple_arrow_return() {
        let diags = lint("jest.fn().mockImplementation(() => 42);");
        assert_eq!(
            diags.len(),
            1,
            "`mockImplementation(() => 42)` should be flagged"
        );
    }

    #[test]
    fn test_allows_mock_return_value() {
        let diags = lint("jest.fn().mockReturnValue(42);");
        assert!(diags.is_empty(), "`mockReturnValue` should not be flagged");
    }

    #[test]
    fn test_allows_promise_resolve_implementation() {
        // This is handled by prefer-mock-promise-shorthand, not this rule
        let diags = lint("jest.fn().mockImplementation(() => Promise.resolve(42));");
        assert!(
            diags.is_empty(),
            "Promise.resolve in mockImplementation should not be flagged by this rule"
        );
    }
}
