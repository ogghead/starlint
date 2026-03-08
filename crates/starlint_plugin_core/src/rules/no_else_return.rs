//! Rule: `no-else-return`
//!
//! Disallow `else` blocks after `return` in `if` statements. If the `if`
//! block always returns, the `else` is unnecessary and the code can be
//! flattened.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags unnecessary `else` blocks after `return`.
#[derive(Debug)]
pub struct NoElseReturn;

impl LintRule for NoElseReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-else-return".to_owned(),
            description: "Disallow `else` blocks after `return` in `if` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        // Must have an else block
        let Some(alternate_id) = if_stmt.alternate else {
            return;
        };

        // The consequent must always return
        if !consequent_always_returns(if_stmt.consequent, ctx) {
            return;
        }

        let source = ctx.source_text();
        let Some(alternate) = ctx.node(alternate_id) else {
            return;
        };
        let alt_span = alternate.span();
        let alt_start = usize::try_from(alt_span.start).unwrap_or(0);
        let alt_end = usize::try_from(alt_span.end).unwrap_or(0);
        let alt_source = source.get(alt_start..alt_end).unwrap_or("");

        // If alternate is a block, extract inner content (strip braces)
        let body_text = if matches!(alternate, AstNode::BlockStatement(_)) {
            alt_source
                .get(1..alt_source.len().saturating_sub(1))
                .unwrap_or("")
                .trim()
        } else {
            alt_source.trim()
        };

        // Replace ` else { ... }` (from consequent end to if_stmt end)
        // with `\n` + the body statements
        let cons_end = ctx
            .node(if_stmt.consequent)
            .map_or(if_stmt.span.end, |n| n.span().end);

        ctx.report(Diagnostic {
            rule_name: "no-else-return".to_owned(),
            message:
                "Unnecessary `else` after `return` — remove the `else` and outdent its contents"
                    .to_owned(),
            span: Span::new(if_stmt.span.start, if_stmt.span.end),
            severity: Severity::Warning,
            help: Some("Remove the `else` wrapper".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Remove the `else` wrapper".to_owned(),
                edits: vec![Edit {
                    span: Span::new(cons_end, if_stmt.span.end),
                    replacement: format!("\n{body_text}"),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Check if a statement always returns.
fn consequent_always_returns(stmt_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(stmt) = ctx.node(stmt_id) else {
        return false;
    };
    match stmt {
        AstNode::ReturnStatement(_) => true,
        AstNode::BlockStatement(block) => block
            .body
            .last()
            .is_some_and(|last_id| matches!(ctx.node(*last_id), Some(AstNode::ReturnStatement(_)))),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoElseReturn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_else_after_return() {
        let diags = lint("function f(x) { if (x) { return 1; } else { return 2; } }");
        assert_eq!(diags.len(), 1, "else after return should be flagged");
    }

    #[test]
    fn test_allows_no_else() {
        let diags = lint("function f(x) { if (x) { return 1; } return 2; }");
        assert!(diags.is_empty(), "no else should not be flagged");
    }

    #[test]
    fn test_allows_no_return_in_if() {
        let diags = lint("function f(x) { if (x) { foo(); } else { bar(); } }");
        assert!(diags.is_empty(), "if without return should not be flagged");
    }
}
