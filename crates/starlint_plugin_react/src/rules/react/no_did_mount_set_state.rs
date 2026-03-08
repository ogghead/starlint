//! Rule: `react/no-did-mount-set-state`
//!
//! Disallow `setState` in `componentDidMount`. Calling `setState` in
//! `componentDidMount` triggers an extra re-render that can cause performance
//! issues and confusing behavior.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `this.setState()` calls inside `componentDidMount`.
#[derive(Debug)]
pub struct NoDidMountSetState;

impl LintRule for NoDidMountSetState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-did-mount-set-state".to_owned(),
            description: "Disallow `setState` in `componentDidMount`".to_owned(),
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

        let Some("componentDidMount") = method_name else {
            return;
        };

        // method.value is a NodeId pointing to a Function node
        let body_span = ctx
            .node(method.value)
            .and_then(|n| n.as_function())
            .and_then(|f| f.body)
            .and_then(|body_id| ctx.node(body_id))
            .map(starlint_ast::AstNode::span);
        let Some(body_span) = body_span else {
            return;
        };

        // Walk the body source range looking for this.setState calls
        let method_start = body_span.start;
        let method_end = body_span.end;
        let source = ctx.source_text();

        // Simple source-text scan for `this.setState` within the method body
        let start_idx = usize::try_from(method_start).unwrap_or(0);
        let end_idx = usize::try_from(method_end).unwrap_or(0);
        let body_source = &source[start_idx..end_idx];
        if body_source.contains("this.setState") {
            ctx.report(Diagnostic {
                rule_name: "react/no-did-mount-set-state".to_owned(),
                message: "Do not use `setState` in `componentDidMount`".to_owned(),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDidMountSetState)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_set_state_in_did_mount() {
        let source = r"
class MyComponent extends React.Component {
    componentDidMount() {
        this.setState({ loaded: true });
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "setState in componentDidMount should be flagged"
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
    fn test_allows_did_mount_without_set_state() {
        let source = r"
class MyComponent extends React.Component {
    componentDidMount() {
        console.log('mounted');
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "componentDidMount without setState should not be flagged"
        );
    }
}
