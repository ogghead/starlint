//! Rule: `jsx-a11y/click-events-have-key-events`
//!
//! Enforce `onClick` is accompanied by `onKeyDown`, `onKeyUp`, or `onKeyPress`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/click-events-have-key-events";

#[derive(Debug)]
pub struct ClickEventsHaveKeyEvents;

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

impl NativeRule for ClickEventsHaveKeyEvents {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `onClick` is accompanied by a keyboard event handler".to_owned(),
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

        if !has_attribute(opening, "onClick") {
            return;
        }

        let has_key_event = has_attribute(opening, "onKeyDown")
            || has_attribute(opening, "onKeyUp")
            || has_attribute(opening, "onKeyPress");

        if !has_key_event {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Elements with `onClick` must have a keyboard event handler (`onKeyDown`, `onKeyUp`, or `onKeyPress`)".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ClickEventsHaveKeyEvents)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_onclick_without_key_event() {
        let diags = lint(r"const el = <div onClick={handleClick}>content</div>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_onclick_with_onkeydown() {
        let diags =
            lint(r"const el = <div onClick={handleClick} onKeyDown={handleKey}>content</div>;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_onclick() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
