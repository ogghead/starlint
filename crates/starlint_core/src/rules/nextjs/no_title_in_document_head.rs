//! Rule: `nextjs/no-title-in-document-head`
//!
//! Forbid `<title>` inside `next/head` `<Head>`. Page titles should be set
//! using the `next/document` `Head` component or the metadata API.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-title-in-document-head";

/// Flags `<title>` elements nested inside `<Head>` in `_document` files.
#[derive(Debug)]
pub struct NoTitleInDocumentHead;

impl LintRule for NoTitleInDocumentHead {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<title>` in `next/document` `<Head>`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Only check _document files
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_stem != "_document" {
            return;
        }

        let AstNode::JSXElement(element) = node else {
            return;
        };

        // Check if this is a <Head> element
        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };
        if opening.name.as_str() != "Head" {
            return;
        }

        // Check children for <title> elements
        for child_id in &*element.children {
            if let Some(AstNode::JSXElement(child_element)) = ctx.node(*child_id) {
                if let Some(AstNode::JSXOpeningElement(child_opening)) =
                    ctx.node(child_element.opening_element)
                {
                    if child_opening.name.as_str() == "title" {
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: "Do not use `<title>` in `_document` `<Head>` -- set page titles in individual pages or use the metadata API".to_owned(),
                            span: Span::new(
                                child_opening.span.start,
                                child_opening.span.end,
                            ),
                            severity: Severity::Warning,
                            help: None,
                            fix: Some(Fix {
                                kind: FixKind::SuggestionFix,
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint_with_path(source: &str, path: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoTitleInDocumentHead)];
        lint_source(source, path, &rules)
    }

    #[test]
    fn test_flags_title_in_document_head() {
        let diags = lint_with_path(
            r"const el = <Head><title>My Page</title></Head>;",
            "pages/_document.tsx",
        );
        assert_eq!(diags.len(), 1, "title in document Head should be flagged");
    }

    #[test]
    fn test_allows_title_in_page_head() {
        let diags = lint_with_path(
            r"const el = <Head><title>My Page</title></Head>;",
            "pages/index.tsx",
        );
        assert!(diags.is_empty(), "title in page Head should pass");
    }

    #[test]
    fn test_allows_meta_in_document_head() {
        let diags = lint_with_path(
            r#"const el = <Head><meta charSet="utf-8" /></Head>;"#,
            "pages/_document.tsx",
        );
        assert!(diags.is_empty(), "meta in document Head should pass");
    }
}
