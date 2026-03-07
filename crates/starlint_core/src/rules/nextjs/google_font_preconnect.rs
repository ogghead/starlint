//! Rule: `nextjs/google-font-preconnect`
//!
//! Enforce preconnect for Google Fonts. `<link>` elements with a Google Fonts
//! `href` should have `rel="preconnect"` to speed up font loading.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "nextjs/google-font-preconnect";

/// Flags `<link>` elements with Google Fonts href that are missing `rel="preconnect"`.
#[derive(Debug)]
pub struct GoogleFontPreconnect;

/// Get string value from a JSX attribute's value node.
fn get_string_value(ctx: &LintContext<'_>, value: Option<NodeId>) -> Option<String> {
    let id = value?;
    let node = ctx.node(id)?;
    if let AstNode::StringLiteral(lit) = node {
        Some(lit.value.clone())
    } else {
        None
    }
}

impl LintRule for GoogleFontPreconnect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce preconnect for Google Fonts".to_owned(),
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

        if opening.name.as_str() != "link" {
            return;
        }

        // Check if href points to Google Fonts
        let has_google_fonts_href = opening.attributes.iter().any(|attr_id| {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
                return false;
            };
            if attr.name.as_str() == "href" {
                if let Some(val) = get_string_value(ctx, attr.value) {
                    return val.contains("fonts.googleapis.com")
                        || val.contains("fonts.gstatic.com");
                }
            }
            false
        });

        if !has_google_fonts_href {
            return;
        }

        // Check for rel="preconnect"
        let has_preconnect = opening.attributes.iter().any(|attr_id| {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
                return false;
            };
            if attr.name.as_str() == "rel" {
                return get_string_value(ctx, attr.value).as_deref() == Some("preconnect");
            }
            false
        });

        if !has_preconnect {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "`<link>` for Google Fonts should have `rel=\"preconnect\"` for faster loading"
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(GoogleFontPreconnect)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_missing_preconnect() {
        let diags = lint(
            r#"const el = <link href="https://fonts.googleapis.com/css" rel="stylesheet" />;"#,
        );
        assert_eq!(diags.len(), 1, "missing preconnect should be flagged");
    }

    #[test]
    fn test_allows_with_preconnect() {
        let diags =
            lint(r#"const el = <link href="https://fonts.gstatic.com" rel="preconnect" />;"#);
        assert!(diags.is_empty(), "link with preconnect should pass");
    }

    #[test]
    fn test_ignores_non_google_fonts_link() {
        let diags =
            lint(r#"const el = <link href="https://example.com/style.css" rel="stylesheet" />;"#);
        assert!(
            diags.is_empty(),
            "non-Google Fonts link should not be flagged"
        );
    }
}
