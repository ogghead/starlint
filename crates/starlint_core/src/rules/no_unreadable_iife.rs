//! Rule: `no-unreadable-iife`
//!
//! Disallow unreadable IIFEs (Immediately Invoked Function Expressions).
//! Traditional IIFEs using `(function() { ... })()` are harder to read than
//! arrow function IIFEs. Arrow IIFEs like `(() => ...)()` are excluded because
//! they are considered more readable.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags non-arrow IIFEs.
#[derive(Debug)]
pub struct NoUnreadableIife;

impl NativeRule for NoUnreadableIife {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unreadable-iife".to_owned(),
            description: "Disallow unreadable IIFEs (non-arrow function IIFEs)".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if the callee is a function expression (possibly wrapped in parens).
        // This covers `(function() {})()` — the outer call with a parenthesized
        // function expression as callee.
        if is_function_iife_callee(&call.callee) {
            ctx.report(Diagnostic {
                rule_name: "no-unreadable-iife".to_owned(),
                message: "IIFE with `function` expression is hard to read; consider using an arrow function".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a non-arrow function expression, unwrapping parens.
///
/// This handles:
/// - `(function() { ... })()` — callee is `ParenthesizedExpression(FunctionExpression)`
/// - `(function() { ... }())` — in this form, the inner call's callee is the
///   `FunctionExpression` directly, and the outer paren wraps the `CallExpression`.
///   However, in oxc's AST the `(function() {}())` form also parses as a
///   `CallExpression` whose callee is a `FunctionExpression` inside parens.
fn is_function_iife_callee(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::FunctionExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => is_function_iife_callee(&paren.expression),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnreadableIife)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_function_iife_outer_parens() {
        let diags = lint("(function() { return 1; })();");
        assert_eq!(
            diags.len(),
            1,
            "function IIFE with outer parens should be flagged"
        );
    }

    #[test]
    fn test_flags_function_iife_inner_parens() {
        let diags = lint("(function() { return 1; }());");
        assert_eq!(
            diags.len(),
            1,
            "function IIFE with inner parens should be flagged"
        );
    }

    #[test]
    fn test_allows_arrow_iife() {
        let diags = lint("(() => 1)();");
        assert!(diags.is_empty(), "arrow IIFE should not be flagged");
    }

    #[test]
    fn test_allows_normal_function_declaration() {
        let diags = lint("function foo() { return 1; }");
        assert!(
            diags.is_empty(),
            "function declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_function_call() {
        let diags = lint("foo();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }

    #[test]
    fn test_flags_named_function_iife() {
        let diags = lint("(function myFunc() { return 1; })();");
        assert_eq!(diags.len(), 1, "named function IIFE should be flagged");
    }
}
