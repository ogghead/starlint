//! Rule: `no-async-await`
//!
//! Flag all async/await usage. Some codebases prefer explicit Promise chains
//! over async/await syntax.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags async functions and `await` expressions.
#[derive(Debug)]
pub struct NoAsyncAwait;

impl LintRule for NoAsyncAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-async-await".to_owned(),
            description: "Disallow async/await".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ArrowFunctionExpression,
            AstNodeType::AwaitExpression,
            AstNodeType::Function,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::Function(func) if func.is_async => {
                ctx.report(Diagnostic {
                    rule_name: "no-async-await".to_owned(),
                    message: "Unexpected async function".to_owned(),
                    span: Span::new(func.span.start, func.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
            AstNode::ArrowFunctionExpression(arrow) if arrow.is_async => {
                ctx.report(Diagnostic {
                    rule_name: "no-async-await".to_owned(),
                    message: "Unexpected async function".to_owned(),
                    span: Span::new(arrow.span.start, arrow.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
            AstNode::AwaitExpression(await_expr) => {
                ctx.report(Diagnostic {
                    rule_name: "no-async-await".to_owned(),
                    message: "Unexpected `await` expression".to_owned(),
                    span: Span::new(await_expr.span.start, await_expr.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_rule_framework::lint_source;
    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAsyncAwait)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_async_function_declaration() {
        let diags = lint("async function foo() {}");
        assert_eq!(
            diags.len(),
            1,
            "async function declaration should be flagged"
        );
    }

    #[test]
    fn test_flags_async_arrow_function() {
        let diags = lint("const f = async () => {};");
        assert_eq!(diags.len(), 1, "async arrow function should be flagged");
    }

    #[test]
    fn test_flags_await_expression() {
        let diags = lint("async function foo() { await bar(); }");
        // Should flag: 1 for async function + 1 for await expression
        assert_eq!(
            diags.len(),
            2,
            "async function and await expression should both be flagged"
        );
    }

    #[test]
    fn test_allows_regular_function() {
        let diags = lint("function foo() {}");
        assert!(diags.is_empty(), "regular function should not be flagged");
    }

    #[test]
    fn test_allows_regular_arrow() {
        let diags = lint("const f = () => {};");
        assert!(
            diags.is_empty(),
            "regular arrow function should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_then() {
        let diags = lint("fetch('/api').then(res => res.json());");
        assert!(diags.is_empty(), "promise chain should not be flagged");
    }
}
