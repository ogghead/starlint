//! Rule: `nextjs/no-img-element`
//!
//! Forbid `<img>` HTML element, use `next/image` instead for optimized
//! image loading with automatic lazy loading, resizing, and format selection.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-img-element";

/// Flags `<img>` elements.
#[derive(Debug)]
pub struct NoImgElement;

impl LintRule for NoImgElement {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<img>` HTML element, use `next/image` instead".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        let is_img = opening.name.as_str() == "img";

        if is_img {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not use `<img>`, use `next/image` `<Image>` instead for optimized images"
                        .to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
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

    starlint_rule_framework::lint_rule_test!(NoImgElement);

    #[test]
    fn test_flags_img_element() {
        let diags = lint(r#"const el = <img src="/photo.jpg" alt="photo" />;"#);
        assert_eq!(diags.len(), 1, "<img> should be flagged");
    }

    #[test]
    fn test_allows_image_component() {
        let diags = lint(r#"const el = <Image src="/photo.jpg" alt="photo" />;"#);
        assert!(diags.is_empty(), "<Image> should not be flagged");
    }

    #[test]
    fn test_allows_other_elements() {
        let diags = lint(r"const el = <div>hello</div>;");
        assert!(diags.is_empty(), "other elements should not be flagged");
    }
}
