//! Rule: `nextjs/no-css-tags`
//!
//! Forbid `<link rel="stylesheet">` tags. In Next.js, CSS should be imported
//! via `import` statements so it can be optimized and code-split.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-css-tags";

/// Flags `<link rel="stylesheet">` elements.
#[derive(Debug)]
pub struct NoCssTags;

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

impl NativeRule for NoCssTags {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<link rel=\"stylesheet\">` tags, use CSS imports instead"
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

        let has_stylesheet_rel = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "rel" {
                    return get_string_value(attr.value.as_ref()) == Some("stylesheet");
                }
            }
            false
        });

        if has_stylesheet_rel {
            ctx.report_warning(
                RULE_NAME,
                "Do not use `<link rel=\"stylesheet\">` -- use CSS `import` statements instead for Next.js optimization",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCssTags)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_stylesheet_link() {
        let diags = lint(r#"const el = <link rel="stylesheet" href="/style.css" />;"#);
        assert_eq!(diags.len(), 1, "stylesheet link should be flagged");
    }

    #[test]
    fn test_allows_preconnect_link() {
        let diags = lint(r#"const el = <link rel="preconnect" href="https://example.com" />;"#);
        assert!(diags.is_empty(), "preconnect link should not be flagged");
    }

    #[test]
    fn test_allows_icon_link() {
        let diags = lint(r#"const el = <link rel="icon" href="/favicon.ico" />;"#);
        assert!(diags.is_empty(), "icon link should not be flagged");
    }
}
