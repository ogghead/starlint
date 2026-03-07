//! Rule: `react/no-danger-with-children`
//!
//! Flag elements with both `children` prop/content and `dangerouslySetInnerHTML`.
//! Using both at the same time is invalid because `dangerouslySetInnerHTML`
//! replaces children, so having both is contradictory and causes runtime errors.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags elements that use both `dangerouslySetInnerHTML` and children.
#[derive(Debug)]
pub struct NoDangerWithChildren;

impl LintRule for NoDangerWithChildren {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-danger-with-children".to_owned(),
            description: "Disallow using `dangerouslySetInnerHTML` together with children"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };

        let mut has_danger = false;
        let mut has_children_prop = false;

        for &attr_id in &*opening.attributes {
            if let Some(AstNode::JSXAttribute(a)) = ctx.node(attr_id) {
                if a.name == "dangerouslySetInnerHTML" {
                    has_danger = true;
                } else if a.name == "children" {
                    has_children_prop = true;
                }
            }
        }

        if !has_danger {
            return;
        }

        let has_child_nodes = !element.children.is_empty();

        if has_children_prop || has_child_nodes {
            ctx.report(Diagnostic {
                rule_name: "react/no-danger-with-children".to_owned(),
                message: "Cannot use `dangerouslySetInnerHTML` and `children` at the same time"
                    .to_owned(),
                span: Span::new(element.span.start, element.span.end),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDangerWithChildren)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_danger_with_child_nodes() {
        let source = r#"var x = <div dangerouslySetInnerHTML={{ __html: "hi" }}>child</div>;"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "dangerouslySetInnerHTML with child nodes should be flagged"
        );
    }

    #[test]
    fn test_flags_danger_with_children_prop() {
        let source =
            r#"var x = <div dangerouslySetInnerHTML={{ __html: "hi" }} children="child" />;"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "dangerouslySetInnerHTML with children prop should be flagged"
        );
    }

    #[test]
    fn test_allows_danger_alone() {
        let source = r#"var x = <div dangerouslySetInnerHTML={{ __html: "hi" }} />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "dangerouslySetInnerHTML alone should not be flagged"
        );
    }

    #[test]
    fn test_allows_children_alone() {
        let source = "var x = <div>hello</div>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "children alone should not be flagged");
    }
}
