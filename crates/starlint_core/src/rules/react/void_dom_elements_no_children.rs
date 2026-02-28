//! Rule: `react/void-dom-elements-no-children`
//!
//! Void DOM elements (`<img>`, `<br>`, `<hr>`, `<input>`, etc.) must not have
//! children or use `dangerouslySetInnerHTML`. These elements cannot contain
//! content in HTML.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXAttributeItem;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// HTML void elements that cannot have children.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Flags void DOM elements that have children or dangerous HTML props.
#[derive(Debug)]
pub struct VoidDomElementsNoChildren;

impl NativeRule for VoidDomElementsNoChildren {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/void-dom-elements-no-children".to_owned(),
            description: "Void DOM elements must not have children".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        // Get the element name; only check lowercase DOM elements
        let Some(tag_name) = element.opening_element.name.get_identifier_name() else {
            return;
        };

        let tag_str = tag_name.as_str();

        // Only check known void elements
        if !VOID_ELEMENTS.contains(&tag_str) {
            return;
        }

        // Check for children
        let has_children = !element.children.is_empty();

        // Check for children or dangerouslySetInnerHTML props
        let has_children_prop = element.opening_element.attributes.iter().any(|attr| {
            if let JSXAttributeItem::Attribute(a) = attr {
                a.is_identifier("children") || a.is_identifier("dangerouslySetInnerHTML")
            } else {
                false
            }
        });

        if has_children || has_children_prop {
            ctx.report_error(
                "react/void-dom-elements-no-children",
                &format!("`<{tag_str}>` is a void element and must not have children"),
                Span::new(element.span.start, element.span.end),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.jsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(VoidDomElementsNoChildren)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

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
