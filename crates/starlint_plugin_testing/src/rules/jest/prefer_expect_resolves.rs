//! Rule: `jest/prefer-expect-resolves`
//!
//! Suggest `expect(promise).resolves.toBe()` over `expect(await promise).toBe()`.
//! Using `.resolves` provides better failure messages and makes the async
//! intent more explicit.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `expect(await promise)` in favor of `expect(promise).resolves`.
#[derive(Debug)]
pub struct PreferExpectResolves;

impl LintRule for PreferExpectResolves {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-expect-resolves".to_owned(),
            description: "Suggest using `expect(...).resolves` instead of `expect(await ...)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `expect(...)` call
        let is_expect = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // Check if the first argument is an await expression
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };
        let Some(AstNode::AwaitExpression(await_expr)) = ctx.node(*first_arg_id) else {
            return;
        };

        // Build fix: `expect(await expr)` -> `await expect(expr).resolves`
        let source = ctx.source_text();
        let inner_span = ctx.node(await_expr.argument).map_or(
            starlint_ast::types::Span::new(0, 0),
            starlint_ast::AstNode::span,
        );
        let inner_text = source
            .get(inner_span.start as usize..inner_span.end as usize)
            .unwrap_or("")
            .to_owned();

        let fix = if inner_text.is_empty() {
            None
        } else {
            // We need the full outer context to figure out what comes after expect(await expr)
            // e.g. `.toBe(1)` — keep that suffix by only replacing the expect call itself
            let replacement = format!("await expect({inner_text}).resolves");
            Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(call.span.start, call.span.end),
                    replacement,
                }],
                is_snippet: false,
            })
        };

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-expect-resolves".to_owned(),
            message: "Use `expect(promise).resolves` instead of `expect(await promise)`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Use `.resolves` matcher instead of awaiting inside `expect()`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferExpectResolves)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_expect_await() {
        let diags = lint("async function t() { expect(await fetchData()).toBe(1); }");
        assert_eq!(diags.len(), 1, "`expect(await ...)` should be flagged");
    }

    #[test]
    fn test_allows_resolves() {
        let diags = lint("async function t() { await expect(fetchData()).resolves.toBe(1); }");
        assert!(
            diags.is_empty(),
            "`.resolves` pattern should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_await_expect() {
        let diags = lint("expect(getValue()).toBe(1);");
        assert!(
            diags.is_empty(),
            "`expect()` without await argument should not be flagged"
        );
    }
}
