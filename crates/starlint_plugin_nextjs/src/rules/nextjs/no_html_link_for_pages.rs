//! Rule: `nextjs/no-html-link-for-pages`
//!
//! Forbid `<a href="/path">` for internal navigation. In Next.js, use the
//! `<Link>` component from `next/link` for client-side routing.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-html-link-for-pages";

/// Flags `<a>` elements with internal `href` paths that should use `<Link>`.
#[derive(Debug)]
pub struct NoHtmlLinkForPages;

/// Get string value from a JSX attribute's value node.
fn get_attr_string_value(
    attr: &starlint_ast::node::JSXAttributeNode,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let value_id = attr.value?;
    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
        Some(lit.value.clone())
    } else {
        None
    }
}

impl LintRule for NoHtmlLinkForPages {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<a href>` for internal navigation, use `<Link>` instead"
                .to_owned(),
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

        if opening.name.as_str() != "a" {
            return;
        }

        // Check if href is an internal path (starts with /)
        let has_internal_href = opening.attributes.iter().any(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() == "href" {
                    if let Some(val) = get_attr_string_value(attr, ctx) {
                        return val.starts_with('/') && !val.starts_with("//");
                    }
                }
            }
            false
        });

        if has_internal_href {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use `<a>` for internal navigation -- use `<Link>` from `next/link` for client-side routing".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(NoHtmlLinkForPages);

    #[test]
    fn test_flags_internal_anchor() {
        let diags = lint(r#"const el = <a href="/about">About</a>;"#);
        assert_eq!(diags.len(), 1, "internal anchor should be flagged");
    }

    #[test]
    fn test_allows_external_anchor() {
        let diags = lint(r#"const el = <a href="https://example.com">External</a>;"#);
        assert!(diags.is_empty(), "external anchor should not be flagged");
    }

    #[test]
    fn test_allows_link_component() {
        let diags = lint(r#"const el = <Link href="/about">About</Link>;"#);
        assert!(diags.is_empty(), "Link component should not be flagged");
    }
}
