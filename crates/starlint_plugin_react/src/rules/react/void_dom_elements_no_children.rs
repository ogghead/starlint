//! Rule: `react/void-dom-elements-no-children`
//!
//! Void DOM elements (`<img>`, `<br>`, `<hr>`, `<input>`, etc.) must not have
//! children or use `dangerouslySetInnerHTML`. These elements cannot contain
//! content in HTML.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// HTML void elements that cannot have children.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Flags void DOM elements that have children or dangerous HTML props.
#[derive(Debug)]
pub struct VoidDomElementsNoChildren;

impl LintRule for VoidDomElementsNoChildren {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/void-dom-elements-no-children".to_owned(),
            description: "Void DOM elements must not have children".to_owned(),
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

        // Get the element name from the opening element
        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };

        let tag_str = opening.name.as_str();

        // Only check known void elements (lowercase DOM elements)
        if !VOID_ELEMENTS.contains(&tag_str) {
            return;
        }

        // Check for children
        let has_children = !element.children.is_empty();

        // Check for children or dangerouslySetInnerHTML props — collect the
        // offending attribute span so we can offer a removal fix.
        let bad_attr_span = opening.attributes.iter().find_map(|attr_id| {
            if let Some(AstNode::JSXAttribute(a)) = ctx.node(*attr_id) {
                if a.name == "children" || a.name == "dangerouslySetInnerHTML" {
                    return Some(Span::new(a.span.start, a.span.end));
                }
            }
            None
        });

        let has_children_prop = bad_attr_span.is_some();

        if has_children || has_children_prop {
            // Only offer a fix when the violation is a removable prop.
            // Child nodes are structural — removing them needs manual review.
            let fix = if has_children {
                None
            } else {
                bad_attr_span.map(|attr_span| {
                    let edit = fix_utils::remove_jsx_attr(ctx.source_text(), attr_span);
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Remove the prop".to_owned(),
                        edits: vec![edit],
                        is_snippet: false,
                    }
                })
            };
            ctx.report(Diagnostic {
                rule_name: "react/void-dom-elements-no-children".to_owned(),
                message: format!("`<{tag_str}>` is a void element and must not have children"),
                span: Span::new(element.span.start, element.span.end),
                severity: Severity::Error,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(VoidDomElementsNoChildren);

    #[test]
    fn test_flags_img_with_children() {
        let source = "var x = <img>child</img>;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "img with children should be flagged");
    }

    #[test]
    fn test_flags_br_with_children_prop() {
        let source = "var x = <br children=\"text\" />;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "br with children prop should be flagged");
    }

    #[test]
    fn test_flags_input_with_dangerous_html() {
        let source = "var x = <input dangerouslySetInnerHTML={{ __html: '' }} />;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "input with dangerouslySetInnerHTML should be flagged"
        );
    }

    #[test]
    fn test_allows_self_closing_img() {
        let source = "var x = <img src=\"a.png\" />;";
        let diags = lint(source);
        assert!(diags.is_empty(), "self-closing img should not be flagged");
    }

    #[test]
    fn test_allows_div_with_children() {
        let source = "var x = <div>hello</div>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "div with children should not be flagged");
    }
}
