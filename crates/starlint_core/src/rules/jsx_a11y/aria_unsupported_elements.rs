//! Rule: `jsx-a11y/aria-unsupported-elements`
//!
//! Forbid `aria-*` and `role` attributes on elements that don't support them.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-unsupported-elements";

/// Elements that do not support ARIA roles or attributes.
const UNSUPPORTED_ELEMENTS: &[&str] = &[
    "meta", "html", "script", "style", "head", "title", "base", "col",
];

#[derive(Debug)]
pub struct AriaUnsupportedElements;

impl NativeRule for AriaUnsupportedElements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Forbid `aria-*` and `role` attributes on elements that don't support them"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if !UNSUPPORTED_ELEMENTS.contains(&element_name) {
            return;
        }

        let has_aria_or_role = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                match &attr.name {
                    JSXAttributeName::Identifier(ident) => {
                        let name = ident.name.as_str();
                        name.starts_with("aria-") || name == "role"
                    }
                    JSXAttributeName::NamespacedName(_) => false,
                }
            } else {
                false
            }
        });

        if has_aria_or_role {
            ctx.report_warning(
                RULE_NAME,
                &format!("`<{element_name}>` does not support ARIA roles or `aria-*` attributes"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AriaUnsupportedElements)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_aria_on_meta() {
        let diags = lint(r#"const el = <meta aria-hidden="true" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_role_on_script() {
        let diags = lint(r#"const el = <script role="button">content</script>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_aria_on_div() {
        let diags = lint(r#"const el = <div aria-label="hello">content</div>;"#);
        assert!(diags.is_empty());
    }
}
