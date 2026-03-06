//! Rule: `no-async-await`
//!
//! Flag all async/await usage. Some codebases prefer explicit Promise chains
//! over async/await syntax.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags async functions and `await` expressions.
#[derive(Debug)]
pub struct NoAsyncAwait;

impl NativeRule for NoAsyncAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-async-await".to_owned(),
            description: "Disallow async/await".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::AwaitExpression,
            AstType::Function,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::Function(func) if func.r#async => {
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
            AstKind::ArrowFunctionExpression(arrow) if arrow.r#async => {
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
            AstKind::AwaitExpression(await_expr) => {
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAsyncAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
