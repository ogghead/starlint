//! Rule: `jsx-a11y/html-has-lang`
//!
//! Enforce `<html>` element has a `lang` attribute.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/html-has-lang";

#[derive(Debug)]
pub struct HtmlHasLang;

impl NativeRule for HtmlHasLang {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `<html>` element has a `lang` attribute".to_owned(),
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

        let is_html = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "html",
            _ => false,
        };

        if !is_html {
            return;
        }

        let has_lang = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "lang",
                    JSXAttributeName::NamespacedName(_) => false,
                }
            } else {
                false
            }
        });

        if !has_lang {
            let source = ctx.source_text();
            let end = usize::try_from(opening.span.end).unwrap_or(0);
            let insert_pos =
                if end > 1 && source.as_bytes().get(end.saturating_sub(2)) == Some(&b'/') {
                    opening.span.end.saturating_sub(2)
                } else {
                    opening.span.end.saturating_sub(1)
                };
            let fix = FixBuilder::new("Add `lang` attribute")
                .edit(fix_utils::insert_before(insert_pos, " lang=\"en\""))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`<html>` elements must have a `lang` attribute".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(HtmlHasLang)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_html_without_lang() {
        let diags = lint(r"const el = <html><body>content</body></html>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_html_with_lang() {
        let diags = lint(r#"const el = <html lang="en"><body>content</body></html>;"#);
        assert!(diags.is_empty());
    }
}
