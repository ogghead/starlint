//! Rule: `react/no-set-state`
//!
//! Disallow usage of `setState`. When using an external state management
//! library (Redux, `MobX`, etc.), `setState` should not be used at all.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags all `this.setState()` and bare `setState()` calls.
#[derive(Debug)]
pub struct NoSetState;

impl LintRule for NoSetState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-set-state".to_owned(),
            description: "Disallow usage of `setState`".to_owned(),
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

        let is_set_state = match ctx.node(call.callee) {
            // this.setState(...)
            Some(AstNode::StaticMemberExpression(member)) => {
                member.property.as_str() == "setState"
                    && ctx
                        .node(member.object)
                        .is_some_and(|n| matches!(n, AstNode::ThisExpression(_)))
            }
            // setState(...)
            Some(AstNode::IdentifierReference(ident)) => ident.name.as_str() == "setState",
            _ => false,
        };

        if is_set_state {
            ctx.report(Diagnostic {
                rule_name: "react/no-set-state".to_owned(),
                message: "Do not use `setState` — manage state with an external store instead"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
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

    starlint_rule_framework::lint_rule_test!(NoSetState);

    #[test]
    fn test_flags_this_set_state() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ count: 1 });
    }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "this.setState should be flagged");
    }

    #[test]
    fn test_flags_bare_set_state() {
        let source = "setState({ count: 1 });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "bare setState call should be flagged");
    }

    #[test]
    fn test_allows_other_method_calls() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.forceUpdate();
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "other method calls should not be flagged");
    }
}
