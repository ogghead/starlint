//! Rule: `jest/prefer-to-have-been-called`
//!
//! Suggest `toHaveBeenCalled()` over `toBe(true)` on mock `.called` property.
//! Using the dedicated matcher provides more descriptive failure messages.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `expect(mock.called).toBe(true)` patterns.
#[derive(Debug)]
pub struct PreferToHaveBeenCalled;

impl LintRule for PreferToHaveBeenCalled {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-have-been-called".to_owned(),
            description: "Suggest using `toHaveBeenCalled()` over `toBe(true)` on `.called`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains(".called") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `.toBe(true)` or `.toBe(false)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        if member.property.as_str() != "toBe" {
            return;
        }

        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };
        let Some(arg_node) = ctx.node(*first_arg_id) else {
            return;
        };
        let is_bool = matches!(arg_node, AstNode::BooleanLiteral(_));
        if !is_bool {
            return;
        }
        let is_true_val = matches!(arg_node, AstNode::BooleanLiteral(b) if b.value);

        // Object must be `expect(...)` call
        let member_object = member.object;
        let Some(AstNode::CallExpression(expect_call)) = ctx.node(member_object) else {
            return;
        };
        let is_expect = matches!(
            ctx.node(expect_call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // First arg of expect() must be `something.called`
        let Some(expect_arg_id) = expect_call.arguments.first() else {
            return;
        };
        let Some(AstNode::StaticMemberExpression(arg_member)) = ctx.node(*expect_arg_id) else {
            return;
        };
        if arg_member.property.as_str() != "called" {
            return;
        }

        // Build fix: extract mock object from `mockFn.called` and boolean value
        let fix = {
            let Some(mock_obj_node) = ctx.node(arg_member.object) else {
                return;
            };
            let mock_obj_span = mock_obj_node.span();
            let source = ctx.source_text();
            #[allow(clippy::as_conversions)]
            let mock_name = source
                .get(mock_obj_span.start as usize..mock_obj_span.end as usize)
                .unwrap_or("");
            let replacement = if is_true_val {
                format!("expect({mock_name}).toHaveBeenCalled()")
            } else {
                format!("expect({mock_name}).not.toHaveBeenCalled()")
            };
            Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(call.span.start, call.span.end),
                    replacement,
                }],
                is_snippet: false,
            })
        };

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-to-have-been-called".to_owned(),
            message: "Use `toHaveBeenCalled()` instead of asserting on `.called` with `toBe()`"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferToHaveBeenCalled);

    #[test]
    fn test_flags_called_to_be_true() {
        let diags = lint("expect(mockFn.called).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(mockFn.called).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_flags_called_to_be_false() {
        let diags = lint("expect(mockFn.called).toBe(false);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(mockFn.called).toBe(false)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_have_been_called() {
        let diags = lint("expect(mockFn).toHaveBeenCalled();");
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalled()` should not be flagged"
        );
    }
}
