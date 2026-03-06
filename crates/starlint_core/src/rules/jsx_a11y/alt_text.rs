//! Rule: `jsx-a11y/alt-text`
//!
//! Enforce alt text on `<img>`, `<area>`, `<input type="image">`, and `<object>`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/alt-text";

/// Elements that require alt text.
const ELEMENTS_REQUIRING_ALT: &[&str] = &["img", "area", "object"];

#[derive(Debug)]
pub struct AltText;

/// Get the element name from a JSX opening element.
fn element_name<'a>(opening: &'a oxc_ast::ast::JSXOpeningElement<'a>) -> Option<&'a str> {
    match &opening.name {
        JSXElementName::Identifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

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

/// Check if an `<input>` element has `type="image"`.
fn is_input_type_image(opening: &oxc_ast::ast::JSXOpeningElement<'_>) -> bool {
    get_attr_string_value(opening, "type") == Some("image")
}

impl NativeRule for AltText {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Enforce alt text on `<img>`, `<area>`, `<input type=\"image\">`, and `<object>`"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let Some(name) = element_name(opening) else {
            return;
        };

        let needs_alt = ELEMENTS_REQUIRING_ALT.contains(&name)
            || (name == "input" && is_input_type_image(opening));

        if !needs_alt {
            return;
        }

        let has_alt = has_attribute(opening, "alt");

        // For <object>, also accept aria-label or aria-labelledby
        let has_aria_label =
            has_attribute(opening, "aria-label") || has_attribute(opening, "aria-labelledby");

        if name == "object" {
            if !has_alt && !has_aria_label {
                let insert_pos = crate::fix_utils::jsx_attr_insert_offset(
                    ctx.source_text(),
                    Span::new(opening.span.start, opening.span.end),
                );
                let fix = FixBuilder::new("Add `aria-label` attribute", FixKind::SuggestionFix)
                    .insert_at(insert_pos, " aria-label=\"${1:object description}\"")
                    .build_snippet();

                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "`<object>` elements must have an `alt`, `aria-label`, or `aria-labelledby` attribute".to_owned(),
                    span: Span::new(opening.span.start, opening.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        } else if !has_alt {
            let insert_pos = crate::fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening.span.start, opening.span.end),
            );
            let fix = FixBuilder::new("Add `alt` attribute", FixKind::SuggestionFix)
                .insert_at(insert_pos, " alt=\"${1:descriptive text}\"")
                .build_snippet();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`<{name}>` elements must have an `alt` attribute"),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AltText)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_img_without_alt() {
        let diags = lint(r#"const el = <img src="foo.png" />;"#);
        assert_eq!(diags.len(), 1, "should flag img without alt");
    }

    #[test]
    fn test_allows_img_with_alt() {
        let diags = lint(r#"const el = <img src="foo.png" alt="A photo" />;"#);
        assert!(diags.is_empty(), "should allow img with alt");
    }

    #[test]
    fn test_flags_input_type_image_without_alt() {
        let diags = lint(r#"const el = <input type="image" src="submit.png" />;"#);
        assert_eq!(diags.len(), 1, "should flag input type=image without alt");
    }
}
