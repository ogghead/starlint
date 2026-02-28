//! Rule: `typescript/return-await`
//!
//! Disallow returning an awaited value when it is unnecessary. In non-try/catch
//! contexts, `return await expr` is redundant because the enclosing `async`
//! function already wraps the return value in a promise. The `await` adds an
//! extra microtask tick with no benefit.
//!
//! Simplified syntax-only version — full checking requires type information.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/return-await";

/// Flags `return await expr` statements where the `await` is redundant.
#[derive(Debug)]
pub struct ReturnAwait;

impl NativeRule for ReturnAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary `return await`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ReturnStatement(ret) = kind else {
            return;
        };

        let Some(arg) = &ret.argument else {
            return;
        };

        if is_await_expression(arg) {
            ctx.report_warning(
                RULE_NAME,
                "Redundant `return await` — the enclosing async function already wraps the \
                 return value in a promise",
                Span::new(ret.span.start, ret.span.end),
            );
        }
    }
}

/// Returns `true` if the expression is an `AwaitExpression`, possibly wrapped
/// in parentheses.
fn is_await_expression(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::AwaitExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => is_await_expression(&paren.expression),
        _ => false,
    }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ReturnAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
    fn test_flags_return_await_parenthesized() {
        let diags = lint("async function f() { return (await fetch('/api')); }");
        assert_eq!(
            diags.len(),
            1,
            "return with parenthesized await should be flagged"
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
