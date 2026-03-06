//! Rule: `nextjs/no-title-in-document-head`
//!
//! Forbid `<title>` inside `next/head` `<Head>`. Page titles should be set
//! using the `next/document` `Head` component or the metadata API.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXChild, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-title-in-document-head";

/// Flags `<title>` elements nested inside `<Head>` in `_document` files.
#[derive(Debug)]
pub struct NoTitleInDocumentHead;

/// Check if a JSX element name matches the given string, handling both
/// lowercase `Identifier` and `PascalCase` `IdentifierReference` variants.
fn is_element_name(name: &JSXElementName<'_>, target: &str) -> bool {
    match name {
        JSXElementName::Identifier(ident) => ident.name.as_str() == target,
        JSXElementName::IdentifierReference(ident) => ident.name.as_str() == target,
        _ => false,
    }
}

impl NativeRule for NoTitleInDocumentHead {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<title>` in `next/document` `<Head>`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Only check _document files
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_stem != "_document" {
            return;
        }

        let AstKind::JSXElement(element) = kind else {
            return;
        };

        // Check if this is a <Head> element
        if !is_element_name(&element.opening_element.name, "Head") {
            return;
        }

        // Check children for <title> elements
        for child in &element.children {
            if let JSXChild::Element(child_element) = child {
                if is_element_name(&child_element.opening_element.name, "title") {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Do not use `<title>` in `_document` `<Head>` -- set page titles in individual pages or use the metadata API".to_owned(),
                        span: Span::new(
                            child_element.opening_element.span.start,
                            child_element.opening_element.span.end,
                        ),
                        severity: Severity::Warning,
                        help: None,
                        fix: Some(Fix {
                            message: "Remove `<title>` element".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(
                                    child_element.span.start,
                                    child_element.span.end,
                                ),
                                replacement: String::new(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTitleInDocumentHead)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_title_in_document_head() {
        let diags = lint_with_path(
            r"const el = <Head><title>My Page</title></Head>;",
            Path::new("pages/_document.tsx"),
        );
        assert_eq!(diags.len(), 1, "title in document Head should be flagged");
    }

    #[test]
    fn test_allows_title_in_page_head() {
        let diags = lint_with_path(
            r"const el = <Head><title>My Page</title></Head>;",
            Path::new("pages/index.tsx"),
        );
        assert!(diags.is_empty(), "title in page Head should pass");
    }

    #[test]
    fn test_allows_meta_in_document_head() {
        let diags = lint_with_path(
            r#"const el = <Head><meta charSet="utf-8" /></Head>;"#,
            Path::new("pages/_document.tsx"),
        );
        assert!(diags.is_empty(), "meta in document Head should pass");
    }
}
