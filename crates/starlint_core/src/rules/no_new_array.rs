//! Rule: `no-new-array` (unicorn)
//!
//! Disallow `new Array()`. Use array literals `[]` or `Array.from()` instead.
//! `new Array(n)` creates a sparse array which can be confusing.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `new Array()` calls.
#[derive(Debug)]
pub struct NoNewArray;

impl LintRule for NoNewArray {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-array".to_owned(),
            description: "Disallow `new Array()` — use `[]` or `Array.from()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let callee_node = ctx.node(new_expr.callee);
        let is_array = matches!(
            callee_node,
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Array"
        );

        if is_array {
            // Remove `new ` prefix: replace whole span with source from callee start
            let callee_start = callee_node.map_or(0, |n| n.span().start as usize);
            let expr_end = new_expr.span.end as usize;
            let source = ctx.source_text();
            let without_new = source.get(callee_start..expr_end).unwrap_or("").to_owned();

            ctx.report(Diagnostic {
                rule_name: "no-new-array".to_owned(),
                message: "Use `[]` or `Array.from()` instead of `new Array()`".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Warning,
                help: Some("Remove `new` keyword".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove `new` keyword".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        replacement: without_new,
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNewArray)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_array() {
        let diags = lint("var a = new Array(10);");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_array_literal() {
        let diags = lint("var a = [];");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_array_from() {
        let diags = lint("var a = Array.from({length: 10});");
        assert!(diags.is_empty());
    }
}
