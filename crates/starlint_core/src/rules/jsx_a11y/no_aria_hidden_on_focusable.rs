//! Rule: `jsx-a11y/no-aria-hidden-on-focusable`
//!
//! Forbid `aria-hidden="true"` on focusable elements.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-aria-hidden-on-focusable";

/// Inherently interactive (focusable) elements.
const INTERACTIVE_ELEMENTS: &[&str] = &["button", "input", "select", "textarea"];

#[derive(Debug)]
pub struct NoAriaHiddenOnFocusable;

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

impl NativeRule for NoAriaHiddenOnFocusable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `aria-hidden=\"true\"` on focusable elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // Check if aria-hidden="true"
        let is_aria_hidden = get_attr_string_value(opening, "aria-hidden") == Some("true");
        if !is_aria_hidden {
            return;
        }

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        // Check if inherently interactive
        let is_interactive = INTERACTIVE_ELEMENTS.contains(&element_name);

        // <a> with href is focusable
        let is_anchor_with_href = element_name == "a" && has_attribute(opening, "href");

        // Any element with tabIndex is focusable
        let has_tabindex = has_attribute(opening, "tabIndex");

        if is_interactive || is_anchor_with_href || has_tabindex {
            ctx.report_warning(
                RULE_NAME,
                "`aria-hidden=\"true\"` must not be set on focusable elements",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAriaHiddenOnFocusable)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_aria_hidden_on_button() {
        let diags = lint(r#"const el = <button aria-hidden="true">click</button>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_aria_hidden_on_anchor_with_href() {
        let diags = lint(r#"const el = <a href="/about" aria-hidden="true">link</a>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_aria_hidden_on_div() {
        let diags = lint(r#"const el = <div aria-hidden="true">content</div>;"#);
        assert!(diags.is_empty());
    }
}
