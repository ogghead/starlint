//! Rule: `nextjs/no-document-import-in-page`
//!
//! Forbid importing `next/document` outside of `pages/_document`.
//! The Document component and its exports are only valid in `_document`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-document-import-in-page";

/// Flags imports of `next/document` outside of `_document` files.
#[derive(Debug)]
pub struct NoDocumentImportInPage;

impl NativeRule for NoDocumentImportInPage {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid importing `next/document` outside of `_document`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source_value = import.source.value.as_str();
        if source_value != "next/document" {
            return;
        }

        // Check if the file is _document
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if file_stem != "_document" {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`next/document` should only be imported in `pages/_document`".to_owned(),
                span: Span::new(import.span.start, import.span.end),
                severity: Severity::Error,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Remove `next/document` import".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(import.span.start, import.span.end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDocumentImportInPage)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_document_import_in_page() {
        let diags = lint_with_path(
            r#"import Document from "next/document";"#,
            Path::new("pages/index.ts"),
        );
        assert_eq!(
            diags.len(),
            1,
            "next/document import in page should be flagged"
        );
    }

    #[test]
    fn test_allows_document_import_in_document() {
        let diags = lint_with_path(
            r#"import Document from "next/document";"#,
            Path::new("pages/_document.ts"),
        );
        assert!(
            diags.is_empty(),
            "next/document import in _document should pass"
        );
    }

    #[test]
    fn test_allows_other_imports() {
        let diags = lint_with_path(
            r#"import Head from "next/head";"#,
            Path::new("pages/index.ts"),
        );
        assert!(diags.is_empty(), "other next imports should not be flagged");
    }
}
