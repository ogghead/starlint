//! Rule: `no-async-promise-executor`
//!
//! Disallow using an async function as a Promise executor. The executor
//! function passed to `new Promise(executor)` should not be `async` because:
//! 1. If the async executor throws, the error will be lost instead of
//!    rejecting the promise.
//! 2. If the async executor returns a value, it's ignored.
//!
//! This is almost always a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `new Promise(async (...) => ...)` and `new Promise(async function(...) {...})`.
#[derive(Debug)]
pub struct NoAsyncPromiseExecutor;

impl LintRule for NoAsyncPromiseExecutor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-async-promise-executor".to_owned(),
            description: "Disallow using an async function as a Promise executor".to_owned(),
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

        // Check if it's `new Promise(...)`
        let Some(AstNode::IdentifierReference(callee)) = ctx.node(new_expr.callee) else {
            return;
        };

        if callee.name.as_str() != "Promise" {
            return;
        }

        // Check first argument is an async function or async arrow
        let Some(&first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        let Some(first_arg) = ctx.node(first_arg_id) else {
            return;
        };

        let (is_async, arg_span) = match first_arg {
            AstNode::Function(func) => (func.is_async, func.span),
            AstNode::ArrowFunctionExpression(arrow) => (arrow.is_async, arrow.span),
            _ => (false, new_expr.span),
        };

        if is_async {
            // Fix: remove the `async` keyword from the executor function/arrow
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                source
                    .get(arg_span.start as usize..arg_span.end as usize)
                    .and_then(|text| {
                        text.find("async").map(|pos| {
                            // Remove "async " (with trailing space)
                            let async_start = arg_span
                                .start
                                .saturating_add(u32::try_from(pos).unwrap_or(0));
                            let async_end = async_start.saturating_add(6); // "async " = 6 chars
                            Fix {
                                kind: FixKind::SuggestionFix,
                                message: "Remove `async` from the Promise executor".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(async_start, async_end),
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }
                        })
                    })
            };

            ctx.report(Diagnostic {
                rule_name: "no-async-promise-executor".to_owned(),
                message: "Promise executor should not be an async function".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: None,
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAsyncPromiseExecutor)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_async_arrow_executor() {
        let diags = lint("new Promise(async (resolve, reject) => { resolve(1); });");
        assert_eq!(diags.len(), 1, "async arrow executor should be flagged");
    }

    #[test]
    fn test_flags_async_function_executor() {
        let diags = lint("new Promise(async function(resolve, reject) { resolve(1); });");
        assert_eq!(diags.len(), 1, "async function executor should be flagged");
    }

    #[test]
    fn test_allows_sync_arrow_executor() {
        let diags = lint("new Promise((resolve, reject) => { resolve(1); });");
        assert!(
            diags.is_empty(),
            "sync arrow executor should not be flagged"
        );
    }

    #[test]
    fn test_allows_sync_function_executor() {
        let diags = lint("new Promise(function(resolve, reject) { resolve(1); });");
        assert!(
            diags.is_empty(),
            "sync function executor should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_promise_async() {
        let diags = lint("new Foo(async () => {});");
        assert!(
            diags.is_empty(),
            "async executor on non-Promise should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_with_no_args() {
        let diags = lint("new Promise();");
        assert!(
            diags.is_empty(),
            "Promise with no args should not be flagged"
        );
    }
}
