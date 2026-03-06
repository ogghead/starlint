//! Rule: `jsx-a11y/anchor-is-valid`
//!
//! Enforce anchors are valid (have href, not `#` or `javascript:`).

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/anchor-is-valid";

#[derive(Debug)]
pub struct AnchorIsValid;

/// Check if an attribute exists on a JSX element.
fn has_attribute(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> bool {
    opening.attributes.iter().any(|item| {
        if let JSXAttributeItem::Attribute(attr) = item {
            match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == name,
                JSXAttributeName::NamespacedName(_) => false,
            }
        } else {
            false
        }
    })
}

/// Get string value of an attribute if it's a string literal.
fn get_attr_string_value<'a>(
    opening: &'a oxc_ast::ast::JSXOpeningElement<'a>,
    attr_name: &str,
) -> Option<&'a str> {
    for item in &opening.attributes {
        if let JSXAttributeItem::Attribute(attr) = item {
            let matches = match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == attr_name,
                JSXAttributeName::NamespacedName(_) => false,
            };
            if matches {
                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    return Some(lit.value.as_str());
                }
            }
        }
    }
    None
}

impl NativeRule for AnchorIsValid {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce anchors are valid (have href, not `#` or `javascript:`)"
                .to_owned(),
            category: Category::Correctness,
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

        let is_anchor = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "a",
            _ => false,
        };
        if !is_anchor {
            return;
        }

        // Check if href exists
        if !has_attribute(opening, "href") {
            let insert_pos = fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening.span.start, opening.span.end),
            );
            let fix = FixBuilder::new("Add `href` attribute")
                .insert_at(insert_pos, " href=\"${1:/path}\"")
                .build_snippet();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Anchors must have an `href` attribute".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
            return;
        }

        // Check for invalid href values
        if let Some(href) = get_attr_string_value(opening, "href") {
            if href == "#" || href.starts_with("javascript:") {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Anchors must have a valid `href` attribute. Avoid `#` or `javascript:` URLs".to_owned(),
                    span: Span::new(opening.span.start, opening.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AnchorIsValid)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_anchor_without_href() {
        let diags = lint(r"const el = <a>link</a>;");
        assert_eq!(diags.len(), 1, "should flag anchor without href");
    }

    #[test]
    fn test_flags_anchor_with_hash_href() {
        let diags = lint(r##"const el = <a href="#">link</a>;"##);
        assert_eq!(diags.len(), 1, "should flag anchor with href='#'");
    }

    #[test]
    fn test_allows_anchor_with_valid_href() {
        let diags = lint(r#"const el = <a href="/about">About</a>;"#);
        assert!(diags.is_empty(), "should allow anchor with valid href");
    }
}
