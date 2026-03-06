//! Rule: `jsx-a11y/no-noninteractive-tabindex`
//!
//! Forbid `tabIndex` on non-interactive elements.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-noninteractive-tabindex";

/// Interactive HTML elements that naturally accept tabIndex.
const INTERACTIVE_ELEMENTS: &[&str] = &[
    "a", "button", "input", "select", "textarea", "details", "summary",
];

/// Non-interactive elements (common static elements).
const NON_INTERACTIVE_ELEMENTS: &[&str] = &[
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "span",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

#[derive(Debug)]
pub struct NoNoninteractiveTabindex;

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

/// Get string value and span of an attribute if it's a string literal.
fn get_attr_string_value_and_span<'a>(
    opening: &'a oxc_ast::ast::JSXOpeningElement<'a>,
    attr_name: &str,
) -> Option<(&'a str, oxc_span::Span)> {
    for item in &opening.attributes {
        if let JSXAttributeItem::Attribute(attr) = item {
            let matches = match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == attr_name,
                JSXAttributeName::NamespacedName(_) => false,
            };
            if matches {
                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    return Some((lit.value.as_str(), attr.span));
                }
            }
        }
    }
    None
}

/// Get the span of a named attribute (regardless of value type).
fn get_attr_span(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    attr_name: &str,
) -> Option<oxc_span::Span> {
    for item in &opening.attributes {
        if let JSXAttributeItem::Attribute(attr) = item {
            let matches = match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == attr_name,
                JSXAttributeName::NamespacedName(_) => false,
            };
            if matches {
                return Some(attr.span);
            }
        }
    }
    None
}

impl NativeRule for NoNoninteractiveTabindex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `tabIndex` on non-interactive elements".to_owned(),
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

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        // Skip interactive elements
        if INTERACTIVE_ELEMENTS.contains(&element_name) {
            return;
        }

        // Only check known non-interactive elements
        if !NON_INTERACTIVE_ELEMENTS.contains(&element_name) {
            return;
        }

        // If element has a role, it may be intentionally interactive
        if has_attribute(opening, "role") {
            return;
        }

        // Check for tabIndex attribute
        let tabindex_info = get_attr_string_value_and_span(opening, "tabIndex");
        if let Some((val, attr_oxc_span)) = tabindex_info {
            let parsed = val.parse::<i32>().unwrap_or(-1);
            // tabIndex="-1" is acceptable (removes from tab order)
            if parsed >= 0 {
                let attr_span = Span::new(attr_oxc_span.start, attr_oxc_span.end);
                let fix = FixBuilder::new("Remove `tabIndex` attribute")
                    .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                    .build();
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`<{element_name}>` is non-interactive and should not have `tabIndex`"
                    ),
                    span: Span::new(opening.span.start, opening.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        } else if let Some(attr_oxc_span) = get_attr_span(opening, "tabIndex") {
            // tabIndex without a value (boolean attribute) defaults to 0
            let attr_span = Span::new(attr_oxc_span.start, attr_oxc_span.end);
            let fix = FixBuilder::new("Remove `tabIndex` attribute")
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`<{element_name}>` is non-interactive and should not have `tabIndex`"
                ),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNoninteractiveTabindex)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_tabindex_on_div() {
        let diags = lint(r#"const el = <div tabIndex="0">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_tabindex_on_button() {
        let diags = lint(r#"const el = <button tabIndex="0">click</button>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_negative_tabindex_on_div() {
        let diags = lint(r#"const el = <div tabIndex="-1">content</div>;"#);
        assert!(diags.is_empty());
    }
}
