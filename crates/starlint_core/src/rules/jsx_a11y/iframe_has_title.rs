//! Rule: `jsx-a11y/iframe-has-title`
//!
//! Enforce `<iframe>` elements have a `title` attribute.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/iframe-has-title";

#[derive(Debug)]
pub struct IframeHasTitle;

impl NativeRule for IframeHasTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `<iframe>` elements have a `title` attribute".to_owned(),
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

        let is_iframe = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "iframe",
            _ => false,
        };

        if !is_iframe {
            return;
        }

        let has_title = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "title",
                    JSXAttributeName::NamespacedName(_) => false,
                }
            } else {
                false
            }
        });

        if !has_title {
            let source = ctx.source_text();
            let end = usize::try_from(opening.span.end).unwrap_or(0);
            let insert_pos =
                if end > 1 && source.as_bytes().get(end.saturating_sub(2)) == Some(&b'/') {
                    opening.span.end.saturating_sub(2)
                } else {
                    opening.span.end.saturating_sub(1)
                };
            let fix = FixBuilder::new("Add `title` attribute")
                .edit(fix_utils::insert_before(insert_pos, " title=\"\""))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`<iframe>` elements must have a `title` attribute".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(IframeHasTitle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_iframe_without_title() {
        let diags = lint(r#"const el = <iframe src="https://example.com" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_iframe_with_title() {
        let diags = lint(r#"const el = <iframe src="https://example.com" title="Example" />;"#);
        assert!(diags.is_empty());
    }
}
