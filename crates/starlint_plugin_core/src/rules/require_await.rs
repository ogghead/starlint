//! Rule: `require-await`
//!
//! Disallow async functions which have no `await` expression. An async
//! function without `await` is likely a mistake — the author probably
//! forgot to await something or doesn't need async.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags async functions that contain no `await` expressions.
#[derive(Debug)]
pub struct RequireAwait;

impl LintRule for RequireAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-await".to_owned(),
            description: "Disallow async functions which have no await expression".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::Function(func) if func.is_async => {
                let resolved_body = func.body.and_then(|body_id| ctx.node(body_id)).map(|n| {
                    let s = n.span();
                    starlint_ast::types::Span::new(s.start, s.end)
                });
                let Some(bspan) = resolved_body else {
                    return;
                };
                let name: Option<String> = func
                    .id
                    .and_then(|id| ctx.node(id))
                    .and_then(|n| n.as_binding_identifier())
                    .map(|bi| bi.name.clone());
                check_for_await(ctx, func.span, bspan, name.as_deref());
            }
            AstNode::ArrowFunctionExpression(arrow) if arrow.is_async => {
                let resolved_body = ctx.node(arrow.body).map(|n| {
                    let s = n.span();
                    starlint_ast::types::Span::new(s.start, s.end)
                });
                let Some(bspan) = resolved_body else {
                    return;
                };
                check_for_await(ctx, arrow.span, bspan, None);
            }
            _ => {}
        }
    }
}

/// Check if the body source text contains `await` and report if not.
fn check_for_await(
    ctx: &mut LintContext<'_>,
    func_span: starlint_ast::types::Span,
    body_span: starlint_ast::types::Span,
    name: Option<&str>,
) {
    let source = ctx.source_text();
    let start = usize::try_from(body_span.start).unwrap_or(0);
    let end = usize::try_from(body_span.end)
        .unwrap_or(0)
        .min(source.len());

    let has_await = source.get(start..end).is_some_and(|s| s.contains("await"));

    if !has_await {
        let fn_name = name.unwrap_or("(anonymous)");
        // Remove `async ` (6 chars) from the start of the function
        let fix = Some(Fix {
            kind: FixKind::SafeFix,
            message: "Remove `async` keyword".to_owned(),
            edits: vec![Edit {
                span: Span::new(func_span.start, func_span.start.saturating_add(6)),
                replacement: String::new(),
            }],
            is_snippet: false,
        });
        ctx.report(Diagnostic {
            rule_name: "require-await".to_owned(),
            message: format!("Async function '{fn_name}' has no 'await' expression"),
            span: Span::new(func_span.start, func_span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireAwait)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_async_without_await() {
        let diags = lint("async function foo() { return 1; }");
        assert_eq!(diags.len(), 1, "async without await should be flagged");
    }

    #[test]
    fn test_allows_async_with_await() {
        let diags = lint("async function foo() { await bar(); }");
        assert!(diags.is_empty(), "async with await should not be flagged");
    }

    #[test]
    fn test_allows_non_async() {
        let diags = lint("function foo() { return 1; }");
        assert!(diags.is_empty(), "non-async should not be flagged");
    }

    #[test]
    fn test_flags_async_arrow_without_await() {
        let diags = lint("const foo = async () => { return 1; };");
        assert_eq!(
            diags.len(),
            1,
            "async arrow without await should be flagged"
        );
    }

    #[test]
    fn test_allows_async_arrow_with_await() {
        let diags = lint("const foo = async () => { await bar(); };");
        assert!(
            diags.is_empty(),
            "async arrow with await should not be flagged"
        );
    }
}
