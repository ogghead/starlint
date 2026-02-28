//! Rule: `typescript/no-unsafe-return`
//!
//! Disallow returning `any` typed values from functions. Returning a value
//! cast to `any` defeats the purpose of the function's return type annotation,
//! allowing callers to receive an untyped value without any compiler warning.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `return expr as any` patterns.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, TSType};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags return statements whose argument is an `as any` assertion.
#[derive(Debug)]
pub struct NoUnsafeReturn;

impl NativeRule for NoUnsafeReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-return".to_owned(),
            description: "Disallow returning `any` typed values from functions".to_owned(),
            category: Category::Correctness,
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

        if is_as_any_return(arg) {
            ctx.report_warning(
                "typescript/no-unsafe-return",
                "Unsafe return — returning an `as any` value defeats the function's return type safety",
                Span::new(ret.span.start, ret.span.end),
            );
        }
    }
}

/// Check whether a return argument is an `as any` cast, unwrapping
/// parenthesized expressions.
fn is_as_any_return(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::TSAsExpression(as_expr) => {
            matches!(&as_expr.type_annotation, TSType::TSAnyKeyword(_))
        }
        Expression::ParenthesizedExpression(paren) => is_as_any_return(&paren.expression),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_as_any() {
        let diags = lint("function f() { return value as any; }");
        assert_eq!(diags.len(), 1, "`return value as any` should be flagged");
    }

    #[test]
    fn test_flags_parenthesized_return_as_any() {
        let diags = lint("function f() { return (value as any); }");
        assert_eq!(
            diags.len(),
            1,
            "parenthesized `return (value as any)` should be flagged"
        );
    }

    #[test]
    fn test_allows_return_as_string() {
        let diags = lint("function f() { return value as string; }");
        assert!(
            diags.is_empty(),
            "`return value as string` should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_return() {
        let diags = lint("function f() { return 42; }");
        assert!(diags.is_empty(), "plain return should not be flagged");
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("function f() { return; }");
        assert!(diags.is_empty(), "empty return should not be flagged");
    }
}
