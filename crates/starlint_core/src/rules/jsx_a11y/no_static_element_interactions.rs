//! Rule: `jsx-a11y/no-static-element-interactions`
//!
//! Forbid event handlers on static elements (`<div>`, `<span>`, etc.) without a role.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-static-element-interactions";

/// Static (non-interactive) HTML elements.
const STATIC_ELEMENTS: &[&str] = &[
    "div",
    "span",
    "section",
    "article",
    "aside",
    "footer",
    "header",
    "main",
    "nav",
    "p",
    "blockquote",
    "pre",
    "figure",
    "figcaption",
    "dd",
    "dl",
    "dt",
    "ul",
    "ol",
    "li",
    "fieldset",
    "table",
    "tbody",
    "thead",
    "tfoot",
    "tr",
    "td",
    "th",
];

/// Event handler attribute names that indicate interactivity.
const EVENT_HANDLERS: &[&str] = &[
    "onClick",
    "onKeyDown",
    "onKeyUp",
    "onKeyPress",
    "onMouseDown",
    "onMouseUp",
    "onDoubleClick",
];

#[derive(Debug)]
pub struct NoStaticElementInteractions;

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

impl NativeRule for NoStaticElementInteractions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid event handlers on static elements without a role".to_owned(),
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

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if !STATIC_ELEMENTS.contains(&element_name) {
            return;
        }

        // If it has a role, it is intentionally interactive
        if has_attribute(opening, "role") {
            return;
        }

        // Check for event handler attributes
        let has_event_handler = EVENT_HANDLERS
            .iter()
            .any(|handler| has_attribute(opening, handler));

        if has_event_handler {
            ctx.report_warning(
                RULE_NAME,
                &format!("`<{element_name}>` with event handlers must have a `role` attribute"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoStaticElementInteractions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_div_with_onclick_no_role() {
        let diags = lint(r"const el = <div onClick={handleClick}>content</div>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_div_with_onclick_and_role() {
        let diags = lint(r#"const el = <div onClick={handleClick} role="button">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_button_with_onclick() {
        let diags = lint(r"const el = <button onClick={handleClick}>click</button>;");
        assert!(diags.is_empty());
    }
}
