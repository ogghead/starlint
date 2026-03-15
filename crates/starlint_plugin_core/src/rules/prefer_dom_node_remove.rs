//! Rule: `prefer-dom-node-remove`
//!
//! Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`.
//! The `.remove()` method is simpler and supported in all modern browsers.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `removeChild()` calls, suggesting `.remove()` instead.
#[derive(Debug)]
pub struct PreferDomNodeRemove;

impl LintRule for PreferDomNodeRemove {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-remove".to_owned(),
            description: "Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "removeChild" {
            return;
        }

        // Extract the child argument to build `child.remove()`
        let call_span = Span::new(call.span.start, call.span.end);
        let fix = if call.arguments.len() == 1 {
            let Some(&arg_id) = call.arguments.first() else {
                return;
            };
            let Some(arg_node) = ctx.node(arg_id) else {
                return;
            };
            let arg_span = arg_node.span();
            let child_text = ctx
                .source_text()
                .get(
                    usize::try_from(arg_span.start).unwrap_or(0)
                        ..usize::try_from(arg_span.end).unwrap_or(0),
                )
                .unwrap_or("")
                .to_owned();
            (!child_text.is_empty()).then(|| Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace with `child.remove()`".to_owned(),
                edits: vec![Edit {
                    span: call_span,
                    replacement: format!("{child_text}.remove()"),
                }],
                is_snippet: false,
            })
        } else {
            None
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-dom-node-remove".to_owned(),
            message: "Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`"
                .to_owned(),
            span: call_span,
            severity: Severity::Warning,
            help: Some("Use `childNode.remove()` instead".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferDomNodeRemove);

    #[test]
    fn test_flags_remove_child() {
        let diags = lint("parent.removeChild(child);");
        assert_eq!(
            diags.len(),
            1,
            "parent.removeChild(child) should be flagged"
        );
    }

    #[test]
    fn test_flags_list_remove_child() {
        let diags = lint("list.removeChild(item);");
        assert_eq!(diags.len(), 1, "list.removeChild(item) should be flagged");
    }

    #[test]
    fn test_allows_remove() {
        let diags = lint("child.remove();");
        assert!(diags.is_empty(), "child.remove() should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("parent.appendChild(child);");
        assert!(
            diags.is_empty(),
            "parent.appendChild(child) should not be flagged"
        );
    }
}
