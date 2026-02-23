//! Rule: `jsx-a11y/img-redundant-alt`
//!
//! Forbid words like "image", "picture", or "photo" in `<img>` alt text.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/img-redundant-alt";

/// Redundant words in alt text for images.
const REDUNDANT_WORDS: &[&str] = &["image", "picture", "photo", "img", "photograph"];

#[derive(Debug)]
pub struct ImgRedundantAlt;

impl LintRule for ImgRedundantAlt {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid redundant words like \"image\" or \"photo\" in `<img>` alt text"
                .to_owned(),
            category: Category::Style,
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

        // opening.name is a String
        if opening.name.as_str() != "img" {
            return;
        }

        for attr_id in &*opening.attributes {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() != "alt" {
                    continue;
                }

                if let Some(value_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                        let alt_lower = lit.value.as_str().to_lowercase();
                        for word in REDUNDANT_WORDS {
                            if alt_lower.contains(word) {
                                ctx.report(Diagnostic {
                                    rule_name: RULE_NAME.to_owned(),
                                    message: format!(
                                        "Redundant alt text. Screen readers already announce `<img>` elements as images. Avoid using \"{word}\""
                                    ),
                                    span: Span::new(opening.span.start, opening.span.end),
                                    severity: Severity::Warning,
                                    help: None,
                                    fix: None,
                                    labels: vec![],
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ImgRedundantAlt)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_redundant_alt() {
        let diags = lint(r#"const el = <img alt="image of a cat" src="cat.png" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_descriptive_alt() {
        let diags = lint(r#"const el = <img alt="A fluffy orange cat" src="cat.png" />;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_photo_in_alt() {
        let diags = lint(r#"const el = <img alt="Photo of sunset" src="sunset.png" />;"#);
        assert_eq!(diags.len(), 1);
    }
}
