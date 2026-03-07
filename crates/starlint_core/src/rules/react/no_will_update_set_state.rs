//! Rule: `react/no-will-update-set-state`
//!
//! Disallow `setState` in `componentWillUpdate`. Calling `setState` in
//! `componentWillUpdate` can cause infinite loops and is a common source
//! of bugs.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `this.setState()` calls inside `componentWillUpdate`.
#[derive(Debug)]
pub struct NoWillUpdateSetState;

impl LintRule for NoWillUpdateSetState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-will-update-set-state".to_owned(),
            description: "Disallow `setState` in `componentWillUpdate`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        let method_name = ctx.node(method.key).and_then(|n| match n {
            AstNode::IdentifierReference(id) => Some(id.name.as_str()),
            AstNode::BindingIdentifier(id) => Some(id.name.as_str()),
            _ => None,
        });

        let Some("componentWillUpdate") = method_name else {
            return;
        };

        let body_span = ctx
            .node(method.value)
            .and_then(|n| n.as_function())
            .and_then(|f| f.body)
            .and_then(|body_id| ctx.node(body_id))
            .map(starlint_ast::AstNode::span);
        let Some(body_span) = body_span else {
            return;
        };

        let source = ctx.source_text();
        let start_idx = usize::try_from(body_span.start).unwrap_or(0);
        let end_idx = usize::try_from(body_span.end).unwrap_or(0);
        let body_source = &source[start_idx..end_idx];
        if body_source.contains("this.setState") {
            ctx.report(Diagnostic {
                rule_name: "react/no-will-update-set-state".to_owned(),
                message: "Do not use `setState` in `componentWillUpdate`".to_owned(),
                span: Span::new(method.span.start, method.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoWillUpdateSetState)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_set_state_in_will_update() {
        let source = r"
class MyComponent extends React.Component {
    componentWillUpdate() {
        this.setState({ updated: true });
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "setState in componentWillUpdate should be flagged"
        );
    }

    #[test]
    fn test_allows_set_state_in_other_methods() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ clicked: true });
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "setState in other methods should not be flagged"
        );
    }

    #[test]
    fn test_allows_will_update_without_set_state() {
        let source = r"
class MyComponent extends React.Component {
    componentWillUpdate() {
        console.log('will update');
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "componentWillUpdate without setState should not be flagged"
        );
    }
}
