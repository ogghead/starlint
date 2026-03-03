//! Rule: `no-async-promise-executor`
//!
//! Disallow using an async function as a Promise executor. The executor
//! function passed to `new Promise(executor)` should not be `async` because:
//! 1. If the async executor throws, the error will be lost instead of
//!    rejecting the promise.
//! 2. If the async executor returns a value, it's ignored.
//!
//! This is almost always a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Promise(async (...) => ...)` and `new Promise(async function(...) {...})`.
#[derive(Debug)]
pub struct NoAsyncPromiseExecutor;

impl NativeRule for NoAsyncPromiseExecutor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-async-promise-executor".to_owned(),
            description: "Disallow using an async function as a Promise executor".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        // Check if it's `new Promise(...)`
        let Expression::Identifier(callee) = &new_expr.callee else {
            return;
        };

        if callee.name.as_str() != "Promise" {
            return;
        }

        // Check first argument is an async function or async arrow
        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        let is_async = match first_arg {
            oxc_ast::ast::Argument::FunctionExpression(func) => func.r#async,
            oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) => arrow.r#async,
            _ => false,
        };

        if is_async {
            ctx.report_error(
                "no-async-promise-executor",
                "Promise executor should not be an async function",
                Span::new(new_expr.span.start, new_expr.span.end),
            );
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAsyncPromiseExecutor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
