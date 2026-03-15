//! Rule: `react/state-in-constructor`
//!
//! State should be initialized in the constructor. Using class property syntax
//! for state initialization (`state = {...}`) is less explicit and can be
//! confusing when mixed with constructor-based initialization.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `state` as a class field property definition.
#[derive(Debug)]
pub struct StateInConstructor;

impl LintRule for StateInConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/state-in-constructor".to_owned(),
            description: "State should be initialized in the constructor".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::PropertyDefinition])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::PropertyDefinition(prop) = node else {
            return;
        };

        // Only flag non-static, non-computed property definitions named "state"
        if prop.is_static || prop.computed {
            return;
        }

        let is_state = ctx
            .node(prop.key)
            .and_then(|n| match n {
                AstNode::IdentifierReference(id) => Some(id.name.as_str() == "state"),
                AstNode::BindingIdentifier(id) => Some(id.name.as_str() == "state"),
                _ => None,
            })
            .unwrap_or(false);

        if is_state {
            ctx.report(Diagnostic {
                rule_name: "react/state-in-constructor".to_owned(),
                message: "State initialization should be in a constructor".to_owned(),
                span: Span::new(prop.span.start, prop.span.end),
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

    starlint_rule_framework::lint_rule_test!(StateInConstructor);

    #[test]
    fn test_flags_state_class_property() {
        let source = r"
class MyComponent extends React.Component {
    state = { count: 0 };
    render() { return null; }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "state as class property should be flagged");
    }

    #[test]
    fn test_allows_state_in_constructor() {
        let source = r"
class MyComponent extends React.Component {
    constructor(props) {
        super(props);
        this.state = { count: 0 };
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "state in constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_class_properties() {
        let source = r"
class MyComponent extends React.Component {
    value = 42;
    render() { return null; }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "other class properties should not be flagged"
        );
    }
}
