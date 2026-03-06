//! Rule: `no-unnecessary-await`
//!
//! Disallow awaiting non-promise values (non-thenables). Awaiting a literal
//! like `await 1`, `await "str"`, `await true`, `await null`, or
//! `await undefined` is pointless — the value is not a thenable and the
//! `await` adds an unnecessary microtask tick.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `await` expressions whose argument is a non-thenable literal.
#[derive(Debug)]
pub struct NoUnnecessaryAwait;

impl NativeRule for NoUnnecessaryAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unnecessary-await".to_owned(),
            description: "Disallow awaiting non-promise values".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AwaitExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AwaitExpression(await_expr) = kind else {
            return;
        };

        if is_non_thenable_literal(&await_expr.argument) {
            let await_span = Span::new(await_expr.span.start, await_expr.span.end);
            let arg_span = Span::new(
                await_expr.argument.span().start,
                await_expr.argument.span().end,
            );
            let arg_text = ctx
                .source_text()
                .get(
                    usize::try_from(arg_span.start).unwrap_or(0)
                        ..usize::try_from(arg_span.end).unwrap_or(0),
                )
                .unwrap_or("")
                .to_owned();
            ctx.report(Diagnostic {
                rule_name: "no-unnecessary-await".to_owned(),
                message: "Unnecessary `await` on a non-thenable value".to_owned(),
                span: await_span,
                severity: Severity::Warning,
                help: Some("Remove the `await` keyword".to_owned()),
                fix: Some(Fix {
                    message: "Remove `await`".to_owned(),
                    edits: vec![Edit {
                        span: await_span,
                        replacement: arg_text,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if the expression is a literal that cannot be a thenable:
/// numeric, string, boolean, null, or the identifier `undefined`.
fn is_non_thenable_literal(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
    ) || is_undefined_identifier(expr)
}

/// Returns `true` if the expression is an identifier named `undefined`.
fn is_undefined_identifier(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(ident) if ident.name == "undefined")
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnnecessaryAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_await_numeric_literal() {
        let diags = lint("async function f() { await 1; }");
        assert_eq!(diags.len(), 1, "await on numeric literal should be flagged");
    }

    #[test]
    fn test_flags_await_string_literal() {
        let diags = lint("async function f() { await \"hello\"; }");
        assert_eq!(diags.len(), 1, "await on string literal should be flagged");
    }

    #[test]
    fn test_flags_await_boolean_literal() {
        let diags = lint("async function f() { await true; }");
        assert_eq!(diags.len(), 1, "await on boolean literal should be flagged");
    }

    #[test]
    fn test_flags_await_null() {
        let diags = lint("async function f() { await null; }");
        assert_eq!(diags.len(), 1, "await on null should be flagged");
    }

    #[test]
    fn test_flags_await_undefined() {
        let diags = lint("async function f() { await undefined; }");
        assert_eq!(diags.len(), 1, "await on undefined should be flagged");
    }

    #[test]
    fn test_allows_await_function_call() {
        let diags = lint("async function f() { await fetch('/api'); }");
        assert!(
            diags.is_empty(),
            "await on function call should not be flagged"
        );
    }

    #[test]
    fn test_allows_await_variable() {
        let diags = lint("async function f() { await promise; }");
        assert!(
            diags.is_empty(),
            "await on variable reference should not be flagged"
        );
    }

    #[test]
    fn test_allows_await_new_promise() {
        let diags = lint("async function f() { await new Promise(resolve => resolve()); }");
        assert!(
            diags.is_empty(),
            "await on new Promise should not be flagged"
        );
    }
}
