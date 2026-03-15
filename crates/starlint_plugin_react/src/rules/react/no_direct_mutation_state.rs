//! Rule: `react/no-direct-mutation-state`
//!
//! Disallow direct mutation of `this.state`. Mutating state directly does not
//! trigger a re-render and leads to stale UI. Always use `setState()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `this.state = ...` assignments.
#[derive(Debug)]
pub struct NoDirectMutationState;

impl LintRule for NoDirectMutationState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-direct-mutation-state".to_owned(),
            description: "Disallow direct mutation of `this.state`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Check if the left side is this.state via a member expression.
        // In starlint_ast, assign.left is a NodeId that resolves to the target.
        if is_this_state_target(assign.left, ctx) {
            ctx.report(Diagnostic {
                rule_name: "react/no-direct-mutation-state".to_owned(),
                message: "Do not mutate `this.state` directly — use `setState()` instead"
                    .to_owned(),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an assignment target (resolved via `NodeId`) is `this.state`.
fn is_this_state_target(target_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(target_id) {
        Some(AstNode::StaticMemberExpression(member)) => {
            member.property.as_str() == "state"
                && ctx
                    .node(member.object)
                    .is_some_and(|n| matches!(n, AstNode::ThisExpression(_)))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoDirectMutationState);

    #[test]
    fn test_flags_direct_state_mutation() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.state = { count: 1 };
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "direct this.state assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_set_state() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ count: 1 });
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "setState call should not be flagged");
    }

    #[test]
    fn test_allows_other_this_assignment() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.value = 42;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "this.value assignment should not be flagged"
        );
    }
}
