//! Rule: `no-promise-executor-return`
//!
//! Disallow returning a value from a Promise executor function. The return
//! value of the executor is ignored, and returning a value is likely a mistake
//! (perhaps the author intended `resolve(value)` instead of `return value`).

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `return <value>` inside Promise executor functions.
#[derive(Debug)]
pub struct NoPromiseExecutorReturn;

impl LintRule for NoPromiseExecutorReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-promise-executor-return".to_owned(),
            description: "Disallow returning a value from a Promise executor".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        // Check if this is `new Promise(...)`
        let Some(AstNode::IdentifierReference(callee)) = ctx.node(new_expr.callee) else {
            return;
        };

        if callee.name.as_str() != "Promise" {
            return;
        }

        let Some(first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        // Get the function body from the executor.
        // We need to collect body_id first, then resolve statements in a separate
        // step, to avoid holding an immutable borrow on `ctx` while passing it mutably.
        let body_id = match ctx.node(*first_arg_id) {
            Some(AstNode::Function(func)) => func.body,
            Some(AstNode::ArrowFunctionExpression(arrow)) => Some(arrow.body),
            _ => None,
        };

        let Some(body_id) = body_id else {
            return;
        };

        let stmts = match ctx.node(body_id) {
            Some(AstNode::FunctionBody(body)) => body.statements.clone(),
            _ => return,
        };

        check_statements_for_value_return(&stmts, ctx);
    }
}

/// Walk statements looking for return statements that have a value.
fn check_statements_for_value_return(stmts: &[NodeId], ctx: &mut LintContext<'_>) {
    for stmt_id in stmts {
        check_statement_for_value_return(*stmt_id, ctx);
    }
}

/// Check a single statement for `return <value>`.
fn check_statement_for_value_return(stmt_id: NodeId, ctx: &mut LintContext<'_>) {
    let Some(stmt) = ctx.node(stmt_id) else {
        return;
    };
    match stmt {
        AstNode::ReturnStatement(ret) => {
            if ret.argument.is_some() {
                let ret_span = Span::new(ret.span.start, ret.span.end);
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with bare `return;`".to_owned(),
                    edits: vec![Edit {
                        span: ret_span,
                        replacement: "return;".to_owned(),
                    }],
                    is_snippet: false,
                });
                ctx.report(Diagnostic {
                    rule_name: "no-promise-executor-return".to_owned(),
                    message: "Return statement in Promise executor is ignored".to_owned(),
                    span: ret_span,
                    severity: Severity::Error,
                    help: Some(
                        "Use `resolve(value)` or `reject(error)` instead of `return`".to_owned(),
                    ),
                    fix,
                    labels: vec![],
                });
            }
        }
        AstNode::BlockStatement(block) => {
            let body = block.body.clone();
            check_statements_for_value_return(&body, ctx);
        }
        AstNode::IfStatement(if_stmt) => {
            let consequent = if_stmt.consequent;
            let alternate = if_stmt.alternate;
            check_statement_for_value_return(consequent, ctx);
            if let Some(alt) = alternate {
                check_statement_for_value_return(alt, ctx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoPromiseExecutorReturn);

    #[test]
    fn test_flags_return_value_in_executor() {
        let diags = lint("new Promise(function(resolve, reject) { return 1; });");
        assert_eq!(diags.len(), 1, "return value in executor should be flagged");
    }

    #[test]
    fn test_flags_return_value_in_arrow_executor() {
        let diags = lint("new Promise((resolve, reject) => { return 1; });");
        assert_eq!(
            diags.len(),
            1,
            "return value in arrow executor should be flagged"
        );
    }

    #[test]
    fn test_allows_bare_return() {
        let diags = lint("new Promise(function(resolve, reject) { resolve(1); return; });");
        assert!(
            diags.is_empty(),
            "bare return in executor should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_return() {
        let diags = lint("new Promise(function(resolve, reject) { resolve(1); });");
        assert!(
            diags.is_empty(),
            "executor without return should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_promise() {
        let diags = lint("new Foo(function() { return 1; });");
        assert!(
            diags.is_empty(),
            "non-Promise constructor should not be flagged"
        );
    }
}
