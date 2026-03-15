//! Rule: `no-new-buffer` (unicorn)
//!
//! Disallow `new Buffer()`. The `Buffer` constructor is deprecated — use
//! `Buffer.from()`, `Buffer.alloc()`, or `Buffer.allocUnsafe()` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Buffer()` calls.
#[derive(Debug)]
pub struct NoNewBuffer;

impl LintRule for NoNewBuffer {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-buffer".to_owned(),
            description: "Disallow `new Buffer()` (deprecated)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let is_buffer = matches!(
            ctx.node(new_expr.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Buffer"
        );

        if is_buffer {
            let source = ctx.source_text();
            // Extract arguments source
            let callee_span = ctx.node(new_expr.callee).map_or(
                starlint_ast::types::Span::new(0, 0),
                starlint_ast::AstNode::span,
            );
            let callee_start = usize::try_from(callee_span.start).unwrap_or(0);
            let expr_end = usize::try_from(new_expr.span.end).unwrap_or(0);
            // Get "Buffer(...)" from callee start to end
            let callee_to_end = source.get(callee_start..expr_end).unwrap_or("");
            // Determine method: alloc for numeric arg, from otherwise
            let method = new_expr.arguments.first().map_or("from", |arg_id| {
                if matches!(ctx.node(*arg_id), Some(AstNode::NumericLiteral(_))) {
                    "alloc"
                } else {
                    "from"
                }
            });
            // Replace "Buffer" in callee_to_end with "Buffer.method"
            let replacement = callee_to_end.replacen("Buffer", &format!("Buffer.{method}"), 1);

            ctx.report(Diagnostic {
                rule_name: "no-new-buffer".to_owned(),
                message: "`new Buffer()` is deprecated — use `Buffer.from()`, `Buffer.alloc()`, or `Buffer.allocUnsafe()`".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: Some(format!("Replace with `Buffer.{method}()`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `Buffer.{method}()`"),
                    edits: vec![Edit {
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        replacement,
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

    starlint_rule_framework::lint_rule_test!(NoNewBuffer);

    #[test]
    fn test_flags_new_buffer() {
        let diags = lint("var b = new Buffer(10);");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_buffer_from() {
        let diags = lint("var b = Buffer.from([1, 2]);");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_buffer_alloc() {
        let diags = lint("var b = Buffer.alloc(10);");
        assert!(diags.is_empty());
    }
}
