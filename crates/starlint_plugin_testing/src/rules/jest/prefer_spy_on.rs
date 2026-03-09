//! Rule: `jest/prefer-spy-on`
//!
//! Suggest `jest.spyOn(obj, 'method')` over `obj.method = jest.fn()`.
//! `spyOn` preserves the original implementation and can be easily restored,
//! while direct assignment loses the original function reference.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `obj.method = jest.fn()` patterns.
#[derive(Debug)]
pub struct PreferSpyOn;

impl LintRule for PreferSpyOn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-spy-on".to_owned(),
            description: "Suggest using `jest.spyOn()` instead of `obj.method = jest.fn()`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("jest.fn") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Left side must be a member expression (obj.method or obj['method'])
        let left_node = ctx.node(assign.left);
        let is_member_target = left_node.is_some_and(|n| {
            matches!(
                n,
                AstNode::StaticMemberExpression(_) | AstNode::ComputedMemberExpression(_)
            )
        });
        if !is_member_target {
            return;
        }

        // Right side must be `jest.fn()` call
        let right_node = ctx.node(assign.right);
        if !right_node.is_some_and(|n| is_jest_fn_call(ctx, n)) {
            return;
        }

        // Build fix for StaticMemberExpression targets
        let fix = left_node.and_then(|n| {
            if let AstNode::StaticMemberExpression(member) = n {
                let source = ctx.source_text();
                let obj_span = ctx.node(member.object)?.span();
                #[allow(clippy::as_conversions)]
                let obj_text = source
                    .get(obj_span.start as usize..obj_span.end as usize)
                    .unwrap_or("");
                let prop_name = &member.property;
                let replacement = format!("jest.spyOn({obj_text}, '{prop_name}')");
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(assign.span.start, assign.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                })
            } else {
                None
            }
        });

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-spy-on".to_owned(),
            message:
                "Use `jest.spyOn(obj, 'method')` instead of assigning `jest.fn()` to a property"
                    .to_owned(),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

/// Check if an `AstNode` is a `jest.fn()` call.
fn is_jest_fn_call(ctx: &LintContext<'_>, node: &AstNode) -> bool {
    let AstNode::CallExpression(call) = node else {
        return false;
    };
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return false;
    };
    let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) else {
        return false;
    };
    obj.name.as_str() == "jest" && member.property.as_str() == "fn"
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferSpyOn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_property_assign_jest_fn() {
        let diags = lint("obj.method = jest.fn();");
        assert_eq!(diags.len(), 1, "`obj.method = jest.fn()` should be flagged");
    }

    #[test]
    fn test_allows_spy_on() {
        let diags = lint("jest.spyOn(obj, 'method');");
        assert!(diags.is_empty(), "`jest.spyOn()` should not be flagged");
    }

    #[test]
    fn test_allows_regular_assignment() {
        let diags = lint("obj.method = function() {};");
        assert!(
            diags.is_empty(),
            "regular function assignment should not be flagged"
        );
    }
}
