//! Rule: `typescript/return-await`
//!
//! Disallow returning an awaited value when it is unnecessary. In non-try/catch
//! contexts, `return await expr` is redundant because the enclosing `async`
//! function already wraps the return value in a promise. The `await` adds an
//! extra microtask tick with no benefit.
//!
//! Simplified syntax-only version — full checking requires type information.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/return-await";

/// Flags `return await expr` statements where the `await` is redundant.
#[derive(Debug)]
pub struct ReturnAwait;

impl LintRule for ReturnAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary `return await`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ReturnStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ReturnStatement(ret) = node else {
            return;
        };

        let Some(arg_id) = ret.argument else {
            return;
        };

        if let Some(inner_text) = get_await_inner_text(arg_id, ctx) {
            let inner_owned = inner_text.to_owned();
            let arg_span = ctx.node(arg_id).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Redundant `return await` — the enclosing async function already wraps the return value in a promise".to_owned(),
                span: Span::new(ret.span.start, ret.span.end),
                severity: Severity::Warning,
                help: Some("Remove the `await` keyword".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove redundant `await`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(arg_span.start, arg_span.end),
                        replacement: inner_owned,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// If the expression is an `AwaitExpression` (possibly parenthesized), return
/// the source text of the inner (awaited) expression.
#[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
fn get_await_inner_text<'s>(expr_id: NodeId, ctx: &'s LintContext<'_>) -> Option<&'s str> {
    let expr = ctx.node(expr_id)?;
    match expr {
        AstNode::AwaitExpression(await_expr) => {
            let inner_span = ctx
                .node(await_expr.argument)
                .map(starlint_ast::AstNode::span)?;
            ctx.source_text()
                .get(inner_span.start as usize..inner_span.end as usize)
        }
        // No ParenthesizedExpression in starlint_ast — skip that case
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ReturnAwait)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_return_await() {
        let diags = lint("async function f() { return await fetch('/api'); }");
        assert_eq!(
            diags.len(),
            1,
            "return await should be flagged as redundant"
        );
    }

    #[test]
    fn test_allows_return_without_await() {
        let diags = lint("async function f() { return fetch('/api'); }");
        assert!(
            diags.is_empty(),
            "return without await should not be flagged"
        );
    }

    #[test]
    fn test_allows_standalone_await() {
        let diags = lint("async function f() { await fetch('/api'); }");
        assert!(
            diags.is_empty(),
            "standalone await (not returned) should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("async function f() { return; }");
        assert!(diags.is_empty(), "empty return should not be flagged");
    }
}
