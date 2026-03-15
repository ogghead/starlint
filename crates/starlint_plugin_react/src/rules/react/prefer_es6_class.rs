//! Rule: `react/prefer-es6-class`
//!
//! Prefer ES6 class over `createReactClass`. The `createReactClass` helper
//! is legacy and ES6 classes are the standard way to define React components.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `createReactClass()` and `React.createClass()` calls.
#[derive(Debug)]
pub struct PreferEs6Class;

impl LintRule for PreferEs6Class {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/prefer-es6-class".to_owned(),
            description: "Prefer ES6 class over `createReactClass`".to_owned(),
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

        let is_create_class = match ctx.node(call.callee) {
            // React.createClass(...)
            Some(AstNode::StaticMemberExpression(member)) => {
                member.property.as_str() == "createClass"
                    && ctx.node(member.object).is_some_and(|n| {
                        matches!(n, AstNode::IdentifierReference(id) if id.name.as_str() == "React")
                    })
            }
            // createReactClass(...)
            Some(AstNode::IdentifierReference(ident)) => ident.name.as_str() == "createReactClass",
            _ => false,
        };

        if is_create_class {
            ctx.report(Diagnostic {
                rule_name: "react/prefer-es6-class".to_owned(),
                message: "Use ES6 class instead of `createReactClass`".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(PreferEs6Class);

    #[test]
    fn test_flags_create_react_class() {
        let source = "var Comp = createReactClass({ render() { return null; } });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "createReactClass should be flagged");
    }

    #[test]
    fn test_flags_react_create_class() {
        let source = "var Comp = React.createClass({ render() { return null; } });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "React.createClass should be flagged");
    }

    #[test]
    fn test_allows_es6_class() {
        let source = r"
class MyComponent extends React.Component {
    render() { return null; }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "ES6 class component should not be flagged"
        );
    }
}
