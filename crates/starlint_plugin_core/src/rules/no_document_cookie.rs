//! Rule: `no-document-cookie` (unicorn)
//!
//! Disallow direct use of `document.cookie`. It's error-prone and hard to
//! debug. Use a cookie library or the Cookie Store API instead.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `document.cookie` usage.
#[derive(Debug)]
pub struct NoDocumentCookie;

impl LintRule for NoDocumentCookie {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-document-cookie".to_owned(),
            description: "Disallow direct use of `document.cookie`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticMemberExpression(member) = node else {
            return;
        };

        if member.property != "cookie" {
            return;
        }

        let is_document = matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(id)) if id.name == "document"
        );

        if is_document {
            ctx.report(Diagnostic {
                rule_name: "no-document-cookie".to_owned(),
                message: "Do not use `document.cookie` directly — use a cookie library or the Cookie Store API".to_owned(),
                span: Span::new(member.span.start, member.span.end),
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

    starlint_rule_framework::lint_rule_test!(NoDocumentCookie);

    #[test]
    fn test_flags_document_cookie_read() {
        let diags = lint("var c = document.cookie;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_document_cookie_write() {
        let diags = lint("document.cookie = 'a=b';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_other_property() {
        let diags = lint("var t = document.title;");
        assert!(diags.is_empty());
    }
}
