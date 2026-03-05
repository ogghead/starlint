//! Rule: `nextjs/no-head-import-in-document`
//!
//! Forbid importing `next/head` in `_document`. The `_document` file should
//! use `Head` from `next/document` instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-head-import-in-document";

/// Flags imports of `next/head` in `_document` files.
#[derive(Debug)]
pub struct NoHeadImportInDocument;

impl NativeRule for NoHeadImportInDocument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid importing `next/head` in `_document`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        if import.source.value.as_str() != "next/head" {
            return;
        }

        // Check if the file is _document
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if file_stem == "_document" {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not import `next/head` in `_document` -- use `Head` from `next/document` instead".to_owned(),
                span: Span::new(import.span.start, import.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoHeadImportInDocument)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_head_import_in_document() {
        let diags = lint_with_path(
            r#"import Head from "next/head";"#,
            Path::new("pages/_document.ts"),
        );
        assert_eq!(
            diags.len(),
            1,
            "next/head import in _document should be flagged"
        );
    }

    #[test]
    fn test_allows_head_import_in_page() {
        let diags = lint_with_path(
            r#"import Head from "next/head";"#,
            Path::new("pages/index.ts"),
        );
        assert!(diags.is_empty(), "next/head import in page should pass");
    }

    #[test]
    fn test_allows_document_import_in_document() {
        let diags = lint_with_path(
            r#"import { Head } from "next/document";"#,
            Path::new("pages/_document.ts"),
        );
        assert!(diags.is_empty(), "next/document import should pass");
    }
}
