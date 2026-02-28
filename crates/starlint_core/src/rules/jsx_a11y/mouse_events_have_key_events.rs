//! Rule: `jsx-a11y/mouse-events-have-key-events`
//!
//! Enforce `onMouseOver`/`onMouseOut` have `onFocus`/`onBlur`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/mouse-events-have-key-events";

#[derive(Debug)]
pub struct MouseEventsHaveKeyEvents;

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

impl NativeRule for MouseEventsHaveKeyEvents {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `onMouseOver`/`onMouseOut` have `onFocus`/`onBlur`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // onMouseOver requires onFocus
        if has_attribute(opening, "onMouseOver") && !has_attribute(opening, "onFocus") {
            ctx.report_warning(
                RULE_NAME,
                "`onMouseOver` must be accompanied by `onFocus` for keyboard accessibility",
                Span::new(opening.span.start, opening.span.end),
            );
        }

        // onMouseOut requires onBlur
        if has_attribute(opening, "onMouseOut") && !has_attribute(opening, "onBlur") {
            ctx.report_warning(
                RULE_NAME,
                "`onMouseOut` must be accompanied by `onBlur` for keyboard accessibility",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MouseEventsHaveKeyEvents)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_mouseover_without_focus() {
        let diags = lint(r"const el = <div onMouseOver={handleOver}>content</div>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_mouseover_with_focus() {
        let diags =
            lint(r"const el = <div onMouseOver={handleOver} onFocus={handleFocus}>content</div>;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_mouseout_without_blur() {
        let diags = lint(r"const el = <div onMouseOut={handleOut}>content</div>;");
        assert_eq!(diags.len(), 1);
    }
}
