//! Rule: `react/button-has-type`
//!
//! Warn when `<button>` elements don't have an explicit `type` attribute.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `<button>` JSX elements missing an explicit `type` attribute.
/// Without an explicit type, buttons default to `type="submit"`, which
/// can cause unexpected form submissions.
#[derive(Debug)]
pub struct ButtonHasType;

impl NativeRule for ButtonHasType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/button-has-type".to_owned(),
            description: "Require explicit type attribute on button elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // Check if element is a `button`
        let is_button = match &opening.name {
            JSXElementName::Identifier(id) => id.name.as_str() == "button",
            _ => false,
        };

        if !is_button {
            return;
        }

        // Check if it has a `type` attribute
        let has_type = opening.attributes.iter().any(|attr| match attr {
            JSXAttributeItem::Attribute(a) => match &a.name {
                JSXAttributeName::Identifier(id) => id.name.as_str() == "type",
                JSXAttributeName::NamespacedName(_) => false,
            },
            JSXAttributeItem::SpreadAttribute(_) => false,
        });

        if !has_type {
            // Fix: insert ` type="button"` after `<button`
            // The opening element name ends right after "button"
            let source = ctx.source_text();
            let tag_start = opening.span.start as usize;
            // Find the end of `<button` — skip `<` then the tag name
            let fix = source.get(tag_start..).and_then(|s| {
                // Find first space, `>`, or `/` after `<button`
                let after_lt = s.strip_prefix('<')?;
                let name_end = after_lt.find([' ', '>', '/'])?;
                let insert_pos =
                    u32::try_from(tag_start.saturating_add(1).saturating_add(name_end)).ok()?;
                Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `type=\"button\"`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(insert_pos, insert_pos),
                        replacement: " type=\"button\"".to_owned(),
                    }],
                    is_snippet: false,
                })
            });

            ctx.report(Diagnostic {
                rule_name: "react/button-has-type".to_owned(),
                message: "Missing explicit `type` attribute on `<button>`. Buttons default to type=\"submit\"".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: Some("Add an explicit `type` attribute".to_owned()),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ButtonHasType)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_button_without_type() {
        let source = "const x = <button>Click</button>;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "button without type should be flagged");
    }

    #[test]
    fn test_allows_button_with_type() {
        let source = r#"const x = <button type="button">Click</button>;"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "button with type should not be flagged");
    }

    #[test]
    fn test_allows_non_button_element() {
        let source = "const x = <div>Hello</div>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "non-button element should not be flagged");
    }
}
