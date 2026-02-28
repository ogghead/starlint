//! Rule: `prefer-top-level-await`
//!
//! Flag immediately-invoked async functions (async IIFEs) at the top level
//! that could use top-level `await` instead. Patterns like
//! `(async () => { await foo(); })()` can be simplified to just `await foo()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags async IIFEs that could use top-level `await`.
#[derive(Debug)]
pub struct PreferTopLevelAwait;

/// Unwrap parenthesized expressions to get the inner expression.
fn unwrap_parens<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    let mut current = expr;
    loop {
        match current {
            Expression::ParenthesizedExpression(paren) => current = &paren.expression,
            _ => return current,
        }
    }
}

/// Check if an expression is an async function expression or async arrow.
fn is_async_iife_callee(expr: &Expression<'_>) -> bool {
    let inner = unwrap_parens(expr);
    match inner {
        Expression::FunctionExpression(func) => func.r#async,
        Expression::ArrowFunctionExpression(arrow) => arrow.r#async,
        _ => false,
    }
}

impl NativeRule for PreferTopLevelAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-top-level-await".to_owned(),
            description: "Prefer top-level `await` over async IIFEs".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if the callee (unwrapping parentheses) is an async function
        // expression or async arrow function expression
        if !is_async_iife_callee(&call.callee) {
            return;
        }

        ctx.report_warning(
            "prefer-top-level-await",
            "Prefer top-level `await` over an immediately-invoked async function",
            Span::new(call.span.start, call.span.end),
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

    /// Helper to lint source code as an ES module.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.mjs")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferTopLevelAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.mjs"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_async_function_iife() {
        let diags = lint("(async function() { await foo(); })();");
        assert_eq!(diags.len(), 1, "async function IIFE should be flagged");
    }

    #[test]
    fn test_flags_async_arrow_iife() {
        let diags = lint("(async () => { await foo(); })();");
        assert_eq!(diags.len(), 1, "async arrow IIFE should be flagged");
    }

    #[test]
    fn test_allows_async_function_declaration() {
        let diags = lint("async function foo() { await bar(); }");
        assert!(
            diags.is_empty(),
            "async function declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_sync_iife() {
        let diags = lint("(function() { })();");
        assert!(diags.is_empty(), "sync IIFE should not be flagged");
    }

    #[test]
    fn test_allows_sync_arrow_iife() {
        let diags = lint("(() => { })();");
        assert!(diags.is_empty(), "sync arrow IIFE should not be flagged");
    }

    #[test]
    fn test_allows_normal_async_call() {
        let diags = lint("doSomething();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
