//! Rule: `jsx-a11y/anchor-has-content`
//!
//! Enforce anchors have content.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/anchor-has-content";

#[derive(Debug)]
pub struct AnchorHasContent;

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

impl NativeRule for AnchorHasContent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce anchors have content".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        let opening = &element.opening_element;

        let is_anchor = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "a",
            _ => false,
        };
        if !is_anchor {
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
            let insert_pos = fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening.span.start, opening.span.end),
            );
            let fix = FixBuilder::new("Add `aria-label` attribute")
                .insert_at(insert_pos, " aria-label=\"${1:link text}\"")
                .build_snippet();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Anchors must have content. Provide child text, `aria-label`, or `aria-labelledby`".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AnchorHasContent)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_self_closing_anchor() {
        let diags = lint(r#"const el = <a href="/about" />;"#);
        assert_eq!(
            diags.len(),
            1,
            "should flag self-closing anchor without content"
        );
    }

    #[test]
    fn test_allows_anchor_with_children() {
        let diags = lint(r#"const el = <a href="/about">About</a>;"#);
        assert!(diags.is_empty(), "should allow anchor with text children");
    }

    #[test]
    fn test_allows_self_closing_anchor_with_aria_label() {
        let diags = lint(r#"const el = <a href="/about" aria-label="About page" />;"#);
        assert!(
            diags.is_empty(),
            "should allow self-closing anchor with aria-label"
        );
    }
}
