//! Rule: `react/no-find-dom-node`
//!
//! Disallow usage of `findDOMNode`. `ReactDOM.findDOMNode` is deprecated and
//! will be removed in a future major version. Use `ref` callbacks instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `findDOMNode()` calls.
#[derive(Debug)]
pub struct NoFindDomNode;

impl LintRule for NoFindDomNode {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-find-dom-node".to_owned(),
            description: "Disallow usage of `findDOMNode`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let is_find_dom_node = match ctx.node(call.callee) {
            // ReactDOM.findDOMNode(...) or any obj.findDOMNode(...)
            Some(AstNode::StaticMemberExpression(member)) => {
                member.property.as_str() == "findDOMNode"
            }
            // findDOMNode(...)
            Some(AstNode::IdentifierReference(ident)) => ident.name.as_str() == "findDOMNode",
            _ => false,
        };

        if is_find_dom_node {
            ctx.report(Diagnostic {
                rule_name: "react/no-find-dom-node".to_owned(),
                message: "`findDOMNode` is deprecated — use `ref` callbacks or `createRef` instead"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
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

    starlint_rule_framework::lint_rule_test!(NoFindDomNode);

    #[test]
    fn test_flags_find_dom_node_call() {
        let source = "var node = findDOMNode(this);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "findDOMNode() should be flagged");
    }

    #[test]
    fn test_flags_react_dom_find_dom_node() {
        let source = "var node = ReactDOM.findDOMNode(this);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "ReactDOM.findDOMNode() should be flagged");
    }

    #[test]
    fn test_allows_other_calls() {
        let source = "var node = document.getElementById('root');";
        let diags = lint(source);
        assert!(diags.is_empty(), "other DOM calls should not be flagged");
    }
}
