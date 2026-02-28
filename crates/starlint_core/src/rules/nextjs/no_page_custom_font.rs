//! Rule: `nextjs/no-page-custom-font`
//!
//! Forbid custom fonts in individual pages. Custom fonts should be loaded
//! in `_document` or `_app` to avoid per-page duplication.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-page-custom-font";

/// Flags `<link>` elements loading custom fonts outside of `_document` or `_app`.
#[derive(Debug)]
pub struct NoPageCustomFont;

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

impl NativeRule for NoPageCustomFont {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid custom fonts in pages, load in `_document` or `_app` instead"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let is_link = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "link",
            _ => false,
        };
        if !is_link {
            return;
        }

        // Check if href points to a font resource
        let has_font_href = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "href" {
                    if let Some(val) = get_string_value(attr.value.as_ref()) {
                        return val.contains("fonts.googleapis.com")
                            || val.contains("fonts.gstatic.com")
                            || std::path::Path::new(val).extension().is_some_and(|ext| {
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
            ctx.report_warning(
                RULE_NAME,
                "Custom fonts should be loaded in `_document` or `_app`, not in individual pages",
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoPageCustomFont)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_font_in_page() {
        let diags = lint_with_path(
            r#"const el = <link href="https://fonts.googleapis.com/css?family=Roboto" />;"#,
            Path::new("pages/index.tsx"),
        );
        assert_eq!(diags.len(), 1, "custom font in page should be flagged");
    }

    #[test]
    fn test_allows_font_in_document() {
        let diags = lint_with_path(
            r#"const el = <link href="https://fonts.googleapis.com/css?family=Roboto" />;"#,
            Path::new("pages/_document.tsx"),
        );
        assert!(diags.is_empty(), "custom font in _document should pass");
    }

    #[test]
    fn test_allows_font_in_app() {
        let diags = lint_with_path(
            r#"const el = <link href="https://fonts.googleapis.com/css?family=Roboto" />;"#,
            Path::new("pages/_app.tsx"),
        );
        assert!(diags.is_empty(), "custom font in _app should pass");
    }
}
