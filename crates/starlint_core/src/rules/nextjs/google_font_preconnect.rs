//! Rule: `nextjs/google-font-preconnect`
//!
//! Enforce preconnect for Google Fonts. `<link>` elements with a Google Fonts
//! `href` should have `rel="preconnect"` to speed up font loading.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/google-font-preconnect";

/// Flags `<link>` elements with Google Fonts href that are missing `rel="preconnect"`.
#[derive(Debug)]
pub struct GoogleFontPreconnect;

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

impl NativeRule for GoogleFontPreconnect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce preconnect for Google Fonts".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
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

        // Check if href points to Google Fonts
        let has_google_fonts_href = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "href" {
                    if let Some(val) = get_string_value(attr.value.as_ref()) {
                        return val.contains("fonts.googleapis.com")
                            || val.contains("fonts.gstatic.com");
                    }
                }
            }
            false
        });

        if !has_google_fonts_href {
            return;
        }

        // Check for rel="preconnect"
        let has_preconnect = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "rel" {
                    return get_string_value(attr.value.as_ref()) == Some("preconnect");
                }
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(GoogleFontPreconnect)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
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
