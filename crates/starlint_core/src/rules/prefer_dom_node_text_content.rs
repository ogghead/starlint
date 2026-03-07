//! Rule: `prefer-dom-node-text-content`
//!
//! Prefer `textContent` over `innerText`. The `innerText` property triggers
//! a reflow and has quirky whitespace behavior. `textContent` is faster and
//! more predictable.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags access to `innerText`, suggesting `textContent` instead.
#[derive(Debug)]
pub struct PreferDomNodeTextContent;

impl LintRule for PreferDomNodeTextContent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-text-content".to_owned(),
            description: "Prefer `textContent` over `innerText`".to_owned(),
            category: Category::Suggestion,
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

        if member.property.as_str() != "innerText" {
            return;
        }

        // property is a String, not a node. Compute the property span from the
        // member span: the property occupies the last `property.len()` bytes.
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let prop_start = member.span.end.saturating_sub(member.property.len() as u32);
        let prop_span = Span::new(prop_start, member.span.end);
        ctx.report(Diagnostic {
            rule_name: "prefer-dom-node-text-content".to_owned(),
            message: "Prefer `textContent` over `innerText` — `innerText` triggers a reflow"
                .to_owned(),
            span: Span::new(member.span.start, member.span.end),
            severity: Severity::Warning,
            help: Some("Replace `innerText` with `textContent`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace `innerText` with `textContent`".to_owned(),
                edits: vec![Edit {
                    span: prop_span,
                    replacement: "textContent".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferDomNodeTextContent)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_inner_text() {
        let diags = lint("var t = el.innerText;");
        assert_eq!(diags.len(), 1, "el.innerText should be flagged");
    }

    #[test]
    fn test_flags_inner_text_assignment() {
        let diags = lint("el.innerText = 'hello';");
        assert_eq!(diags.len(), 1, "el.innerText assignment should be flagged");
    }

    #[test]
    fn test_allows_text_content() {
        let diags = lint("var t = el.textContent;");
        assert!(diags.is_empty(), "el.textContent should not be flagged");
    }

    #[test]
    fn test_allows_inner_html() {
        let diags = lint("var h = el.innerHTML;");
        assert!(diags.is_empty(), "el.innerHTML should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_property() {
        let diags = lint("var v = el.style;");
        assert!(diags.is_empty(), "el.style should not be flagged");
    }
}
