//! Rule: `react/iframe-missing-sandbox`
//!
//! Warn when `<iframe>` elements don't have a `sandbox` attribute.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags `<iframe>` JSX elements that lack a `sandbox` attribute.
/// The `sandbox` attribute restricts iframe capabilities and is an
/// important security measure.
#[derive(Debug)]
pub struct IframeMissingSandbox;

impl NativeRule for IframeMissingSandbox {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/iframe-missing-sandbox".to_owned(),
            description: "Require sandbox attribute on iframe elements".to_owned(),
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

        // Check if element is an `iframe`
        let is_iframe = match &opening.name {
            JSXElementName::Identifier(id) => id.name.as_str() == "iframe",
            _ => false,
        };

        if !is_iframe {
            return;
        }

        // Check for `sandbox` attribute
        let has_sandbox = opening.attributes.iter().any(|attr| match attr {
            JSXAttributeItem::Attribute(a) => match &a.name {
                JSXAttributeName::Identifier(id) => id.name.as_str() == "sandbox",
                JSXAttributeName::NamespacedName(_) => false,
            },
            JSXAttributeItem::SpreadAttribute(_) => false,
        });

        if !has_sandbox {
            // Insert `sandbox=""` before the closing `>` or `/>`
            let source = ctx.source_text();
            let end = usize::try_from(opening.span.end).unwrap_or(0);
            let insert_pos =
                if end > 1 && source.as_bytes().get(end.saturating_sub(2)) == Some(&b'/') {
                    // Self-closing: insert before `/>`
                    opening.span.end.saturating_sub(2)
                } else {
                    // Regular: insert before `>`
                    opening.span.end.saturating_sub(1)
                };
            let fix = FixBuilder::new("Add `sandbox` attribute")
                .edit(fix_utils::insert_before(insert_pos, " sandbox=\"\""))
                .build();
            ctx.report(Diagnostic {
                rule_name: "react/iframe-missing-sandbox".to_owned(),
                message: "`<iframe>` elements should have a `sandbox` attribute for security"
                    .to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(IframeMissingSandbox)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_iframe_without_sandbox() {
        let source = r#"const x = <iframe src="https://example.com" />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "iframe without sandbox should be flagged");
    }

    #[test]
    fn test_allows_iframe_with_sandbox() {
        let source = r#"const x = <iframe src="https://example.com" sandbox="allow-scripts" />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "iframe with sandbox should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_iframe_element() {
        let source = "const x = <div />;";
        let diags = lint(source);
        assert!(diags.is_empty(), "non-iframe element should not be flagged");
    }
}
