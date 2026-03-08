//! Rule: `curly`
//!
//! Require braces around the body of control flow statements
//! (`if`, `else`, `for`, `while`, `do`). Omitting braces can lead
//! to bugs when adding statements to the body later.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags control flow statements whose body is not a block statement.
#[derive(Debug)]
pub struct Curly;

/// Check if a node is a block statement (has curly braces).
fn is_block(ctx: &LintContext<'_>, id: NodeId) -> bool {
    ctx.node(id)
        .is_some_and(|n| matches!(n, AstNode::BlockStatement(_)))
}

/// Get the span of a node by ID.
fn node_span(ctx: &LintContext<'_>, id: NodeId) -> starlint_ast::types::Span {
    ctx.node(id).map_or(
        starlint_ast::types::Span::new(0, 0),
        starlint_ast::AstNode::span,
    )
}

impl LintRule for Curly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "curly".to_owned(),
            description: "Require curly braces for all control flow".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::DoWhileStatement,
            AstNodeType::ForInStatement,
            AstNodeType::ForOfStatement,
            AstNodeType::ForStatement,
            AstNodeType::IfStatement,
            AstNodeType::WhileStatement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::IfStatement(stmt) => {
                if !is_block(ctx, stmt.consequent) {
                    let body = node_span(ctx, stmt.consequent);
                    report_curly_fix(ctx, "Expected { after 'if' condition", stmt.span, body);
                }
                if let Some(alt_id) = stmt.alternate {
                    // Don't flag `else if` — only flag `else` without braces
                    let is_block_or_if = ctx.node(alt_id).is_some_and(|n| {
                        matches!(n, AstNode::BlockStatement(_) | AstNode::IfStatement(_))
                    });
                    if !is_block_or_if {
                        let body = node_span(ctx, alt_id);
                        report_curly_fix(ctx, "Expected { after 'else'", stmt.span, body);
                    }
                }
            }
            AstNode::ForStatement(stmt) => {
                if !is_block(ctx, stmt.body) {
                    let body = node_span(ctx, stmt.body);
                    report_curly_fix(ctx, "Expected { after 'for'", stmt.span, body);
                }
            }
            AstNode::ForInStatement(stmt) => {
                if !is_block(ctx, stmt.body) {
                    let body = node_span(ctx, stmt.body);
                    report_curly_fix(ctx, "Expected { after 'for-in'", stmt.span, body);
                }
            }
            AstNode::ForOfStatement(stmt) => {
                if !is_block(ctx, stmt.body) {
                    let body = node_span(ctx, stmt.body);
                    report_curly_fix(ctx, "Expected { after 'for-of'", stmt.span, body);
                }
            }
            AstNode::WhileStatement(stmt) => {
                if !is_block(ctx, stmt.body) {
                    let body = node_span(ctx, stmt.body);
                    report_curly_fix(ctx, "Expected { after 'while' condition", stmt.span, body);
                }
            }
            AstNode::DoWhileStatement(stmt) => {
                if !is_block(ctx, stmt.body) {
                    let body = node_span(ctx, stmt.body);
                    report_curly_fix(ctx, "Expected { after 'do'", stmt.span, body);
                }
            }
            _ => {}
        }
    }
}

/// Report a curly-brace fix by wrapping the body statement in `{ ... }`.
fn report_curly_fix(
    ctx: &mut LintContext<'_>,
    message: &str,
    stmt_span: starlint_ast::types::Span,
    body_span: starlint_ast::types::Span,
) {
    let source = ctx.source_text();
    let start = usize::try_from(body_span.start).unwrap_or(0);
    let end = usize::try_from(body_span.end).unwrap_or(start);
    let body_text = source.get(start..end).unwrap_or_default().to_owned();

    ctx.report(Diagnostic {
        rule_name: "curly".to_owned(),
        message: message.to_owned(),
        span: Span::new(stmt_span.start, stmt_span.end),
        severity: Severity::Warning,
        help: Some("Wrap in curly braces".to_owned()),
        fix: Some(Fix {
            kind: FixKind::SafeFix,
            message: "Wrap in curly braces".to_owned(),
            edits: vec![Edit {
                span: Span::new(body_span.start, body_span.end),
                replacement: format!("{{ {body_text} }}"),
            }],
            is_snippet: false,
        }),
        labels: vec![],
    });
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(Curly)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_if_without_braces() {
        let diags = lint("if (true) return;");
        assert_eq!(diags.len(), 1, "if without braces should be flagged");
    }

    #[test]
    fn test_allows_if_with_braces() {
        let diags = lint("if (true) { return; }");
        assert!(diags.is_empty(), "if with braces should not be flagged");
    }

    #[test]
    fn test_flags_else_without_braces() {
        let diags = lint("if (true) { return; } else return;");
        assert_eq!(diags.len(), 1, "else without braces should be flagged");
    }

    #[test]
    fn test_allows_else_if() {
        let diags = lint("if (a) { return; } else if (b) { return; }");
        assert!(diags.is_empty(), "else if should not be flagged");
    }

    #[test]
    fn test_flags_while_without_braces() {
        let diags = lint("while (true) break;");
        assert_eq!(diags.len(), 1, "while without braces should be flagged");
    }

    #[test]
    fn test_flags_for_without_braces() {
        let diags = lint("for (var i = 0; i < 10; i++) console.log(i);");
        assert_eq!(diags.len(), 1, "for without braces should be flagged");
    }
}
