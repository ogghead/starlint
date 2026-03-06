//! Rule: `typescript/await-thenable`
//!
//! Disallow awaiting non-thenable values. Flags `await` expressions where
//! the argument is a literal value (string, number, boolean, null, undefined,
//! array literal, or object literal) that can never be a thenable.
//!
//! Simplified syntax-only version — full checking requires type information.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/await-thenable";

/// Flags `await` expressions whose argument is a non-thenable literal value.
#[derive(Debug)]
pub struct AwaitThenable;

impl NativeRule for AwaitThenable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow awaiting non-thenable values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AwaitExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AwaitExpression(await_expr) = kind else {
            return;
        };

        if is_non_thenable(&await_expr.argument) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Unexpected `await` of a non-thenable value — this has no effect"
                    .to_owned(),
                span: Span::new(await_expr.span.start, await_expr.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if the expression is a literal or structural value that
/// cannot be a thenable: numeric, string, boolean, null, undefined, array
/// literal, or object literal.
fn is_non_thenable(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
    ) || is_undefined_identifier(expr)
}

/// Returns `true` if the expression is the identifier `undefined`.
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AwaitThenable)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_await_string_literal() {
        let diags = lint("async function f() { await \"hello\"; }");
        assert_eq!(diags.len(), 1, "await on string literal should be flagged");
    }

    #[test]
    fn test_flags_await_number_literal() {
        let diags = lint("async function f() { await 42; }");
        assert_eq!(diags.len(), 1, "await on number literal should be flagged");
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
    fn test_flags_await_array_literal() {
        let diags = lint("async function f() { await [1, 2, 3]; }");
        assert_eq!(diags.len(), 1, "await on array literal should be flagged");
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
}
