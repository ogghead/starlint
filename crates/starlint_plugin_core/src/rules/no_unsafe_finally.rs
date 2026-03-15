//! Rule: `no-unsafe-finally`
//!
//! Disallow control flow statements in `finally` blocks. `return`, `throw`,
//! `break`, and `continue` in a `finally` block silently discard any exception
//! or return value from the `try`/`catch` blocks, leading to confusing behavior.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags control flow statements (`return`, `throw`, `break`, `continue`)
/// inside `finally` blocks.
#[derive(Debug)]
pub struct NoUnsafeFinally;

impl LintRule for NoUnsafeFinally {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unsafe-finally".to_owned(),
            description: "Disallow control flow statements in finally blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TryStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TryStatement(try_stmt) = node else {
            return;
        };

        let Some(finalizer_id) = try_stmt.finalizer else {
            return;
        };

        // finalizer is a BlockStatement node
        let Some(AstNode::BlockStatement(block)) = ctx.node(finalizer_id) else {
            return;
        };

        let stmt_ids: Vec<NodeId> = block.body.to_vec();
        check_statements_for_control_flow(&stmt_ids, ctx);
    }
}

/// Scan statements for control flow that would discard try/catch results.
fn check_statements_for_control_flow(stmt_ids: &[NodeId], ctx: &mut LintContext<'_>) {
    for stmt_id in stmt_ids {
        check_statement_for_control_flow(*stmt_id, ctx);
    }
}

/// Check a single statement for unsafe control flow.
fn check_statement_for_control_flow(stmt_id: NodeId, ctx: &mut LintContext<'_>) {
    match ctx.node(stmt_id) {
        Some(AstNode::ReturnStatement(ret)) => {
            let span = ret.span;
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `return` in finally block".to_owned(),
                span: Span::new(span.start, span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Some(AstNode::ThrowStatement(throw)) => {
            let span = throw.span;
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `throw` in finally block".to_owned(),
                span: Span::new(span.start, span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Some(AstNode::BreakStatement(brk)) => {
            let span = brk.span;
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `break` in finally block".to_owned(),
                span: Span::new(span.start, span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Some(AstNode::ContinueStatement(cont)) => {
            let span = cont.span;
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `continue` in finally block".to_owned(),
                span: Span::new(span.start, span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Some(AstNode::BlockStatement(block)) => {
            let body_ids: Vec<NodeId> = block.body.to_vec();
            check_statements_for_control_flow(&body_ids, ctx);
        }
        Some(AstNode::IfStatement(if_stmt)) => {
            let consequent = if_stmt.consequent;
            let alternate = if_stmt.alternate;
            check_statement_for_control_flow(consequent, ctx);
            if let Some(alt) = alternate {
                check_statement_for_control_flow(alt, ctx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnsafeFinally);

    #[test]
    fn test_flags_return_in_finally() {
        let diags = lint("try {} finally { return 1; }");
        assert_eq!(diags.len(), 1, "return in finally should be flagged");
    }

    #[test]
    fn test_flags_throw_in_finally() {
        let diags = lint("try {} finally { throw new Error(); }");
        assert_eq!(diags.len(), 1, "throw in finally should be flagged");
    }

    #[test]
    fn test_flags_break_in_finally() {
        let diags = lint("A: try {} finally { break A; }");
        assert_eq!(diags.len(), 1, "break in finally should be flagged");
    }

    #[test]
    fn test_allows_no_finally() {
        let diags = lint("try { return 1; } catch (e) {}");
        assert!(
            diags.is_empty(),
            "try without finally should not be flagged"
        );
    }

    #[test]
    fn test_allows_safe_finally() {
        let diags = lint("try {} finally { console.log('done'); }");
        assert!(diags.is_empty(), "safe finally should not be flagged");
    }

    #[test]
    fn test_allows_return_in_catch() {
        let diags = lint("try {} catch (e) { return 1; } finally {}");
        assert!(
            diags.is_empty(),
            "return in catch (not finally) should not be flagged"
        );
    }

    #[test]
    fn test_flags_continue_in_finally() {
        let diags = lint("while(true) { try {} finally { continue; } }");
        assert_eq!(diags.len(), 1, "continue in finally should be flagged");
    }

    #[test]
    fn test_flags_multiple_control_flow_in_finally() {
        let diags = lint("try {} finally { return 1; throw new Error(); }");
        assert_eq!(
            diags.len(),
            2,
            "multiple control flow statements in finally should each be flagged"
        );
    }

    #[test]
    fn test_flags_control_flow_in_nested_block() {
        let diags = lint("try {} finally { { return 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "return inside nested block in finally should be flagged"
        );
    }

    #[test]
    fn test_flags_control_flow_in_if_consequent() {
        let diags = lint("try {} finally { if (x) { return 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "return in if consequent inside finally should be flagged"
        );
    }

    #[test]
    fn test_flags_control_flow_in_if_else() {
        let diags = lint("try {} finally { if (x) { return 1; } else { throw e; } }");
        assert_eq!(
            diags.len(),
            2,
            "control flow in both if branches inside finally should be flagged"
        );
    }

    #[test]
    fn test_nested_try_inside_finally_not_flagged() {
        let diags = lint("try {} finally { try { return 1; } catch(e) {} }");
        // The inner try/catch handles the return, so the outer finally is safe
        assert!(
            diags.is_empty(),
            "return in nested try/catch inside finally should not be flagged"
        );
    }

    #[test]
    fn test_try_with_catch_and_finally() {
        let diags = lint("try { foo(); } catch (e) { bar(); } finally { return 1; }");
        assert_eq!(
            diags.len(),
            1,
            "return in finally with both catch and finally should be flagged"
        );
    }

    #[test]
    fn test_empty_finally_block() {
        let diags = lint("try { foo(); } finally {}");
        assert!(
            diags.is_empty(),
            "empty finally block should not be flagged"
        );
    }
}
