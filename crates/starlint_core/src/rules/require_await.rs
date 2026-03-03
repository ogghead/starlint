//! Rule: `require-await`
//!
//! Disallow async functions which have no `await` expression. An async
//! function without `await` is likely a mistake — the author probably
//! forgot to await something or doesn't need async.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags async functions that contain no `await` expressions.
#[derive(Debug)]
pub struct RequireAwait;

impl NativeRule for RequireAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-await".to_owned(),
            description: "Disallow async functions which have no await expression".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrowFunctionExpression, AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::Function(func) if func.r#async => {
                let Some(body) = &func.body else { return };
                check_for_await(
                    ctx,
                    func.span,
                    body.span,
                    func.id.as_ref().map(|id| id.name.as_str()),
                );
            }
            AstKind::ArrowFunctionExpression(arrow) if arrow.r#async => {
                check_for_await(ctx, arrow.span, arrow.body.span, None);
            }
            _ => {}
        }
    }
}

/// Check if the body source text contains `await` and report if not.
fn check_for_await(
    ctx: &mut crate::rule::NativeLintContext<'_>,
    func_span: oxc_span::Span,
    body_span: oxc_span::Span,
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
        ctx.report_warning(
            "require-await",
            &format!("Async function '{fn_name}' has no 'await' expression"),
            Span::new(func_span.start, func_span.end),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
