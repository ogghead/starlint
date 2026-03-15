//! Rule: `nextjs/no-page-custom-font`
//!
//! Forbid custom fonts in individual pages. Custom fonts should be loaded
//! in `_document` or `_app` to avoid per-page duplication.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::jsx_utils::get_string_value;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-page-custom-font";

/// Flags `<link>` elements loading custom fonts outside of `_document` or `_app`.
#[derive(Debug)]
pub struct NoPageCustomFont;

impl LintRule for NoPageCustomFont {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid custom fonts in pages, load in `_document` or `_app` instead"
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

        if opening.name.as_str() != "link" {
            return;
        }

        // Check if href points to a font resource
        let has_font_href = opening.attributes.iter().any(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() == "href" {
                    if let Some(val) = get_string_value(ctx, attr.value) {
                        return val.contains("fonts.googleapis.com")
                            || val.contains("fonts.gstatic.com")
                            || std::path::Path::new(val.as_str())
                                .extension()
                                .is_some_and(|ext| {
                                    ext.eq_ignore_ascii_case("woff")
                                        || ext.eq_ignore_ascii_case("woff2")
                                        || ext.eq_ignore_ascii_case("ttf")
                                        || ext.eq_ignore_ascii_case("otf")
                                });
                    }
                }
            }
            false
        });

        if !has_font_href {
            return;
        }

        // Check if the file is _document or _app
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if file_stem != "_document" && file_stem != "_app" {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Custom fonts should be loaded in `_document` or `_app`, not in individual pages".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(NoPageCustomFont);

    #[test]
    fn test_flags_font_in_page() {
        let diags =
            lint(r#"const el = <link href="https://fonts.googleapis.com/css?family=Roboto" />;"#);
        assert_eq!(diags.len(), 1, "custom font in page should be flagged");
    }

    #[test]
    fn test_allows_non_font_link() {
        let diags = lint(r#"const el = <link href="/style.css" />;"#);
        assert!(diags.is_empty(), "non-font link should not be flagged");
    }

    #[test]
    fn test_allows_no_href() {
        let diags = lint(r#"const el = <link rel="icon" />;"#);
        assert!(diags.is_empty(), "link without href should not be flagged");
    }
}
