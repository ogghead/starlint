//! Rule: `jest/prefer-to-have-been-called-times`
//!
//! Suggest `toHaveBeenCalledTimes(n)` over `expect(mock.mock.calls.length).toBe(n)`.
//! The dedicated matcher provides clearer failure messages showing the actual
//! call count.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `expect(mock.mock.calls.length).toBe(n)` patterns.
#[derive(Debug)]
pub struct PreferToHaveBeenCalledTimes;

impl LintRule for PreferToHaveBeenCalledTimes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-have-been-called-times".to_owned(),
            description: "Suggest using `toHaveBeenCalledTimes()` instead of asserting on `.mock.calls.length`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains(".calls.length") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `.toBe(n)` or `.toEqual(n)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let method = member.property.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        // Object must be `expect(...)` call
        let Some(AstNode::CallExpression(expect_call)) = ctx.node(member.object) else {
            return;
        };
        let is_expect = matches!(
            ctx.node(expect_call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // The argument to expect() should end in `.calls.length` or `.length`
        // and contain `mock` somewhere in the chain.
        let Some(&expect_arg_id) = expect_call.arguments.first() else {
            return;
        };

        if is_mock_calls_length(expect_arg_id, ctx) {
            // Build fix: extract mock object and count argument
            let fix = {
                let mock_obj_span = extract_mock_object(expect_arg_id, ctx);
                let count_arg_id = call.arguments.first().copied();
                match (mock_obj_span, count_arg_id) {
                    (Some(obj_span), Some(count_id)) => {
                        let source = ctx.source_text();
                        let mock_name = source
                            .get(obj_span.start as usize..obj_span.end as usize)
                            .unwrap_or("");
                        let count_span = ctx.node(count_id).map_or(
                            starlint_ast::types::Span::EMPTY,
                            starlint_ast::AstNode::span,
                        );
                        let count_text = source
                            .get(count_span.start as usize..count_span.end as usize)
                            .unwrap_or("");
                        let replacement =
                            format!("expect({mock_name}).toHaveBeenCalledTimes({count_text})");
                        Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        })
                    }
                    _ => None,
                }
            };

            ctx.report(Diagnostic {
                rule_name: "jest/prefer-to-have-been-called-times".to_owned(),
                message:
                    "Use `toHaveBeenCalledTimes()` instead of asserting on `.mock.calls.length`"
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

/// Extract the root mock object span from a `x.mock.calls.length` or `x.calls.length` chain.
fn extract_mock_object(
    expr_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<starlint_ast::types::Span> {
    let Some(AstNode::StaticMemberExpression(length_member)) = ctx.node(expr_id) else {
        return None;
    };
    let Some(AstNode::StaticMemberExpression(calls_member)) = ctx.node(length_member.object) else {
        return None;
    };
    match ctx.node(calls_member.object)? {
        AstNode::StaticMemberExpression(mock_member) => ctx
            .node(mock_member.object)
            .map(starlint_ast::AstNode::span),
        AstNode::IdentifierReference(id) => Some(id.span),
        _ => None,
    }
}

/// Check if an expression matches `x.mock.calls.length` or `x.calls.length`
/// patterns commonly used to check mock call counts.
fn is_mock_calls_length(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    // Must end in `.length`
    let Some(AstNode::StaticMemberExpression(length_member)) = ctx.node(expr_id) else {
        return false;
    };
    if length_member.property.as_str() != "length" {
        return false;
    }

    // Next level should be `.calls`
    let Some(AstNode::StaticMemberExpression(calls_member)) = ctx.node(length_member.object) else {
        return false;
    };
    if calls_member.property.as_str() != "calls" {
        return false;
    }

    // Optionally `.mock` but at minimum there should be an object
    match ctx.node(calls_member.object) {
        Some(AstNode::StaticMemberExpression(mock_member)) => {
            mock_member.property.as_str() == "mock"
        }
        // Also match `mockFn.calls.length` directly
        Some(AstNode::IdentifierReference(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferToHaveBeenCalledTimes)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_mock_calls_length() {
        let diags = lint("expect(mockFn.mock.calls.length).toBe(2);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(mockFn.mock.calls.length).toBe(2)` should be flagged"
        );
    }

    #[test]
    fn test_flags_calls_length_directly() {
        let diags = lint("expect(spy.calls.length).toBe(1);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(spy.calls.length).toBe(1)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_have_been_called_times() {
        let diags = lint("expect(mockFn).toHaveBeenCalledTimes(2);");
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledTimes()` should not be flagged"
        );
    }
}
