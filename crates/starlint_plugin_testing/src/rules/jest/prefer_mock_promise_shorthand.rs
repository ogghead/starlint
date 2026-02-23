//! Rule: `jest/prefer-mock-promise-shorthand`
//!
//! Suggest `jest.fn().mockResolvedValue(x)` over
//! `jest.fn().mockImplementation(() => Promise.resolve(x))`. The shorthand
//! methods are more readable and clearly express intent.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.mockImplementation(() => Promise.resolve(x))` patterns.
#[derive(Debug)]
pub struct PreferMockPromiseShorthand;

impl LintRule for PreferMockPromiseShorthand {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-mock-promise-shorthand".to_owned(),
            description: "Suggest using `mockResolvedValue`/`mockRejectedValue` shorthand"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("mockImplementation") || source_text.contains("mockReturnValue"))
            && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `.mockImplementation(...)` or `.mockReturnValue(...)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let method = member.property.clone();
        if method != "mockImplementation" && method != "mockReturnValue" {
            return;
        }

        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        let member_object = member.object;
        let call_span = call.span;

        // For mockImplementation: check if the argument is an arrow/function
        // that returns Promise.resolve or Promise.reject
        if method == "mockImplementation" {
            let return_expr_id = match ctx.node(first_arg_id) {
                Some(AstNode::ArrowFunctionExpression(arrow)) => {
                    if arrow.expression {
                        get_single_expression_body(arrow.body, ctx)
                    } else {
                        get_single_return_expression(arrow.body, ctx)
                    }
                }
                Some(AstNode::Function(func)) => {
                    func.body.and_then(|b| get_single_return_expression(b, ctx))
                }
                _ => None,
            };

            let Some(ret_id) = return_expr_id else {
                return;
            };
            if let Some(promise_method) = is_promise_call(ret_id, ctx) {
                let suggestion = match promise_method.as_str() {
                    "resolve" => "mockResolvedValue",
                    "reject" => "mockRejectedValue",
                    _ => return,
                };
                let fix =
                    build_mock_shorthand_fix(call_span, member_object, suggestion, ret_id, ctx);
                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-mock-promise-shorthand".to_owned(),
                    message: format!(
                        "Use `.{suggestion}()` instead of `.mockImplementation(() => Promise.{promise_method}(...))`"
                    ),
                    span: Span::new(call_span.start, call_span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace with `.{suggestion}()`")),
                    fix,
                    labels: vec![],
                });
            }
        } else if method == "mockReturnValue" {
            // Check if the argument is `Promise.resolve(x)` or `Promise.reject(x)`
            if let Some(promise_method) = is_promise_call(first_arg_id, ctx) {
                let suggestion = match promise_method.as_str() {
                    "resolve" => "mockResolvedValue",
                    "reject" => "mockRejectedValue",
                    _ => return,
                };
                let fix = build_mock_shorthand_fix(
                    call_span,
                    member_object,
                    suggestion,
                    first_arg_id,
                    ctx,
                );
                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-mock-promise-shorthand".to_owned(),
                    message: format!(
                        "Use `.{suggestion}()` instead of `.mockReturnValue(Promise.{promise_method}(...))`"
                    ),
                    span: Span::new(call_span.start, call_span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace with `.{suggestion}()`")),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

/// Build fix: replace with `.mockResolvedValue(x)` or `.mockRejectedValue(x)`.
#[allow(clippy::as_conversions)]
fn build_mock_shorthand_fix(
    call_span: starlint_ast::types::Span,
    member_object: NodeId,
    suggestion: &str,
    promise_expr_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<Fix> {
    let source = ctx.source_text();
    // Extract the object before `.mockImplementation(...)` / `.mockReturnValue(...)`
    let obj_span = ctx.node(member_object)?.span();
    let obj_text = source.get(obj_span.start as usize..obj_span.end as usize)?;

    // Extract the argument from Promise.resolve(x) / Promise.reject(x)
    let Some(AstNode::CallExpression(promise_call)) = ctx.node(promise_expr_id) else {
        return None;
    };
    let inner_arg_text = promise_call.arguments.first().and_then(|&a| {
        let sp = ctx.node(a)?.span();
        source.get(sp.start as usize..sp.end as usize)
    });
    let arg_text = inner_arg_text.unwrap_or("");

    let replacement = format!("{obj_text}.{suggestion}({arg_text})");
    Some(Fix {
        kind: FixKind::SafeFix,
        message: format!("Replace with `.{suggestion}()`"),
        edits: vec![Edit {
            span: Span::new(call_span.start, call_span.end),
            replacement,
        }],
        is_snippet: false,
    })
}

/// Check if an expression is `Promise.resolve(...)` or `Promise.reject(...)`.
/// Returns the method name ("resolve" or "reject") if matched.
fn is_promise_call(expr_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return None;
    };
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return None;
    };
    let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) else {
        return None;
    };
    if obj.name.as_str() != "Promise" {
        return None;
    }
    let method = member.property.clone();
    (method == "resolve" || method == "reject").then_some(method)
}

/// Get the single expression from an arrow function expression body.
fn get_single_expression_body(body_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
        return None;
    };
    let &stmt_id = body.statements.first()?;
    if let Some(AstNode::ExpressionStatement(es)) = ctx.node(stmt_id) {
        Some(es.expression)
    } else {
        None
    }
}

/// Get the expression from a function body with a single return statement.
fn get_single_return_expression(body_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
        return None;
    };
    if body.statements.len() != 1 {
        return None;
    }
    let &stmt_id = body.statements.first()?;
    if let Some(AstNode::ReturnStatement(ret)) = ctx.node(stmt_id) {
        ret.argument
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferMockPromiseShorthand)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_mock_implementation_promise_resolve() {
        let diags = lint("jest.fn().mockImplementation(() => Promise.resolve(42));");
        assert_eq!(
            diags.len(),
            1,
            "mockImplementation with Promise.resolve should be flagged"
        );
    }

    #[test]
    fn test_flags_mock_return_value_promise_reject() {
        let diags = lint("jest.fn().mockReturnValue(Promise.reject(new Error('fail')));");
        assert_eq!(
            diags.len(),
            1,
            "mockReturnValue with Promise.reject should be flagged"
        );
    }

    #[test]
    fn test_allows_mock_resolved_value() {
        let diags = lint("jest.fn().mockResolvedValue(42);");
        assert!(
            diags.is_empty(),
            "`mockResolvedValue` should not be flagged"
        );
    }
}
