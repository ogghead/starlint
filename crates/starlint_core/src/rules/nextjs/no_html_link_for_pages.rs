//! Rule: `nextjs/no-html-link-for-pages`
//!
//! Forbid `<a href="/path">` for internal navigation. In Next.js, use the
//! `<Link>` component from `next/link` for client-side routing.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-html-link-for-pages";

/// Flags `<a>` elements with internal `href` paths that should use `<Link>`.
#[derive(Debug)]
pub struct NoHtmlLinkForPages;

/// Get string value from a JSX attribute value.
fn get_string_value<'a>(value: Option<&'a JSXAttributeValue<'a>>) -> Option<&'a str> {
    match value {
        Some(JSXAttributeValue::StringLiteral(lit)) => Some(lit.value.as_str()),
        _ => None,
    }
}

/// Get the attribute name as a string.
fn attr_name<'a>(name: &'a JSXAttributeName<'a>) -> &'a str {
    match name {
        JSXAttributeName::Identifier(ident) => ident.name.as_str(),
        JSXAttributeName::NamespacedName(ns) => ns.name.name.as_str(),
    }
}

impl NativeRule for NoHtmlLinkForPages {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<a href>` for internal navigation, use `<Link>` instead"
                .to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let is_anchor = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "a",
            _ => false,
        };
        if !is_anchor {
            return;
        }

        // Check if href is an internal path (starts with /)
        let has_internal_href = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "href" {
                    if let Some(val) = get_string_value(attr.value.as_ref()) {
                        return val.starts_with('/') && !val.starts_with("//");
                    }
                }
            }
            false
        });

        if has_internal_href {
            ctx.report_warning(
                RULE_NAME,
                "Do not use `<a>` for internal navigation -- use `<Link>` from `next/link` for client-side routing",
                Span::new(opening.span.start, opening.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoHtmlLinkForPages)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

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
