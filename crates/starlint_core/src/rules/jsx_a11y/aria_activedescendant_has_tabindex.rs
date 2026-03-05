//! Rule: `jsx-a11y/aria-activedescendant-has-tabindex`
//!
//! Enforce elements with `aria-activedescendant` are tabbable.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-activedescendant-has-tabindex";

/// Interactive elements that are naturally tabbable.
const INTERACTIVE_ELEMENTS: &[&str] = &["input", "select", "textarea", "button", "a"];

#[derive(Debug)]
pub struct AriaActivedescendantHasTabindex;

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

impl NativeRule for AriaActivedescendantHasTabindex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce elements with `aria-activedescendant` are tabbable".to_owned(),
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

        if !has_attribute(opening, "aria-activedescendant") {
            return;
        }

        // Check if it is an interactive element
        let is_interactive = match &opening.name {
            JSXElementName::Identifier(ident) => {
                INTERACTIVE_ELEMENTS.contains(&ident.name.as_str())
            }
            _ => false,
        };

        if is_interactive {
            return;
        }

        // Non-interactive: must have tabIndex
        let has_tabindex = has_attribute(opening, "tabIndex");
        let tabindex_val = get_attr_string_value(opening, "tabIndex");
        let is_negative = tabindex_val
            .and_then(|v| v.parse::<i32>().ok())
            .is_some_and(|n| n < 0);

        if !has_tabindex || is_negative {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "An element with `aria-activedescendant` must be tabbable. Add `tabIndex`"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AriaActivedescendantHasTabindex)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_div_with_activedescendant_no_tabindex() {
        let diags = lint(r#"const el = <div aria-activedescendant="item-1">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_input_with_activedescendant() {
        let diags = lint(r#"const el = <input aria-activedescendant="item-1" />;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_div_with_activedescendant_and_tabindex() {
        let diags =
            lint(r#"const el = <div aria-activedescendant="item-1" tabIndex="0">content</div>;"#);
        assert!(diags.is_empty());
    }
}
