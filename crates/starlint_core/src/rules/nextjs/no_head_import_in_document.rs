//! Rule: `nextjs/no-head-import-in-document`
//!
//! Forbid importing `next/head` in `_document`. The `_document` file should
//! use `Head` from `next/document` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-head-import-in-document";

/// Flags imports of `next/head` in `_document` files.
#[derive(Debug)]
pub struct NoHeadImportInDocument;

impl LintRule for NoHeadImportInDocument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid importing `next/head` in `_document`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        if import.source.as_str() != "next/head" {
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
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Remove `next/head` import".to_owned(),
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
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint_with_path(
        source: &str,
        path: &str,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoHeadImportInDocument)];
        lint_source(source, path, &rules)
    }

    #[test]
    fn test_flags_head_import_in_document() {
        let diags = lint_with_path(r#"import Head from "next/head";"#, "pages/_document.ts");
        assert_eq!(
            diags.len(),
            1,
            "next/head import in _document should be flagged"
        );
    }

    #[test]
    fn test_allows_head_import_in_page() {
        let diags = lint_with_path(r#"import Head from "next/head";"#, "pages/index.ts");
        assert!(diags.is_empty(), "next/head import in page should pass");
    }

    #[test]
    fn test_allows_document_import_in_document() {
        let diags = lint_with_path(
            r#"import { Head } from "next/document";"#,
            "pages/_document.ts",
        );
        assert!(diags.is_empty(), "next/document import should pass");
    }
}
