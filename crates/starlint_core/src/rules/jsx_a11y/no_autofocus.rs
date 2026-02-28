//! Rule: `jsx-a11y/no-autofocus`
//!
//! Forbid `autoFocus` attribute.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-autofocus";

#[derive(Debug)]
pub struct NoAutofocus;

impl NativeRule for NoAutofocus {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `autoFocus` attribute".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let has_autofocus = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "autoFocus",
                    JSXAttributeName::NamespacedName(_) => false,
                }
            } else {
                false
            }
        });

        if has_autofocus {
            ctx.report_warning(
                RULE_NAME,
                "Do not use `autoFocus`. It can reduce usability and accessibility for users",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAutofocus)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_autofocus() {
        let diags = lint(r"const el = <input autoFocus />;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_without_autofocus() {
        let diags = lint(r"const el = <input />;");
        assert!(diags.is_empty());
    }
}
