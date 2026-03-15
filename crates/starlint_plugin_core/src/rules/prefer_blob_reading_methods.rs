//! Rule: `prefer-blob-reading-methods`
//!
//! Prefer `Blob` reading methods (`blob.text()`, `blob.arrayBuffer()`,
//! `blob.stream()`) over using `FileReader`. The `Blob` API is simpler,
//! promise-based, and avoids the callback-based `FileReader` pattern.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new FileReader()` — prefer `Blob` reading methods instead.
#[derive(Debug)]
pub struct PreferBlobReadingMethods;

impl LintRule for PreferBlobReadingMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-blob-reading-methods".to_owned(),
            description: "Prefer `Blob` reading methods over `FileReader`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let is_file_reader = matches!(
            ctx.node(new_expr.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "FileReader"
        );

        if is_file_reader {
            ctx.report(Diagnostic {
                    rule_name: "prefer-blob-reading-methods".to_owned(),
                    message: "Prefer `Blob` reading methods (`blob.text()`, `blob.arrayBuffer()`, `blob.stream()`) over `FileReader`".to_owned(),
                    span: Span::new(new_expr.span.start, new_expr.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferBlobReadingMethods);

    #[test]
    fn test_flags_new_filereader() {
        let diags = lint("var reader = new FileReader();");
        assert_eq!(diags.len(), 1, "new FileReader() should be flagged");
    }

    #[test]
    fn test_allows_blob_text() {
        let diags = lint("blob.text();");
        assert!(diags.is_empty(), "blob.text() should not be flagged");
    }

    #[test]
    fn test_allows_blob_array_buffer() {
        let diags = lint("blob.arrayBuffer();");
        assert!(diags.is_empty(), "blob.arrayBuffer() should not be flagged");
    }

    #[test]
    fn test_allows_blob_stream() {
        let diags = lint("blob.stream();");
        assert!(diags.is_empty(), "blob.stream() should not be flagged");
    }

    #[test]
    fn test_allows_other_new_expression() {
        let diags = lint("var x = new Map();");
        assert!(diags.is_empty(), "new Map() should not be flagged");
    }
}
