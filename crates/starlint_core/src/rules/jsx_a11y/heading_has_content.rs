//! Rule: `jsx-a11y/heading-has-content`
//!
//! Enforce heading elements (`h1`-`h6`) have content.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/heading-has-content";

/// Heading element names.
const HEADINGS: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6"];

#[derive(Debug)]
pub struct HeadingHasContent;

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

impl NativeRule for HeadingHasContent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce heading elements (`h1`-`h6`) have content".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        let opening = &element.opening_element;

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if !HEADINGS.contains(&element_name) {
            return;
        }

        // If the element has children, it has content
        if !element.children.is_empty() {
            return;
        }

        // Check for aria-label or aria-labelledby as alternative content
        let has_accessible_content =
            has_attribute(opening, "aria-label") || has_attribute(opening, "aria-labelledby");

        if !has_accessible_content {
            ctx.report_warning(
                RULE_NAME,
                &format!("`<{element_name}>` must have content. Provide child text, `aria-label`, or `aria-labelledby`"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(HeadingHasContent)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_heading() {
        let diags = lint(r"const el = <h1 />;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_heading_with_children() {
        let diags = lint(r"const el = <h1>Title</h1>;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_heading_with_aria_label() {
        let diags = lint(r#"const el = <h1 aria-label="Title" />;"#);
        assert!(diags.is_empty());
    }
}
