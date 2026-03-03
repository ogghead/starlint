//! Rule: `prefer-type-error` (unicorn)
//!
//! Prefer throwing `TypeError` in type-checking `if` statements.
//! When checking the type of a value, throwing a `TypeError` is more
//! appropriate than a generic `Error`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, Statement, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags throw statements in type-checking blocks that throw `Error`
/// instead of `TypeError`.
#[derive(Debug)]
pub struct PreferTypeError;

impl NativeRule for PreferTypeError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-type-error".to_owned(),
            description: "Prefer TypeError for type checking".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        // Check if the condition is a type check (typeof, instanceof)
        if !is_type_check(&if_stmt.test) {
            return;
        }

        // Check if the body throws a generic Error
        if throws_generic_error(&if_stmt.consequent) {
            ctx.report_warning(
                "prefer-type-error",
                "Use `new TypeError()` instead of `new Error()` for type checks",
                Span::new(if_stmt.span.start, if_stmt.span.end),
            );
        }
    }
}

/// Check if an expression is a type check (typeof x === '...' or x instanceof Y).
fn is_type_check(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::BinaryExpression(bin) => {
            // typeof x === '...'
            let left_is_typeof = matches!(
                &bin.left,
                Expression::UnaryExpression(u) if u.operator == UnaryOperator::Typeof
            );
            let right_is_typeof = matches!(
                &bin.right,
                Expression::UnaryExpression(u) if u.operator == UnaryOperator::Typeof
            );
            // x instanceof Y
            let is_instanceof = matches!(bin.operator, oxc_ast::ast::BinaryOperator::Instanceof);

            left_is_typeof || right_is_typeof || is_instanceof
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            is_type_check(&unary.argument)
        }
        Expression::LogicalExpression(logical) => {
            is_type_check(&logical.left) || is_type_check(&logical.right)
        }
        Expression::ParenthesizedExpression(paren) => is_type_check(&paren.expression),
        _ => false,
    }
}

/// Check if a statement throws `new Error(...)` (not `new TypeError(...)`).
fn throws_generic_error(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::BlockStatement(block) => {
            block.body.len() == 1 && block.body.first().is_some_and(|s| throws_generic_error(s))
        }
        Statement::ThrowStatement(throw) => {
            matches!(&throw.argument, Expression::NewExpression(new_expr)
                if matches!(&new_expr.callee, Expression::Identifier(id) if id.name == "Error")
            )
        }
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferTypeError)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_typeof_with_error() {
        let diags = lint("if (typeof x !== 'string') { throw new Error('msg'); }");
        assert_eq!(diags.len(), 1, "typeof check with Error should be flagged");
    }

    #[test]
    fn test_allows_typeof_with_type_error() {
        let diags = lint("if (typeof x !== 'string') { throw new TypeError('msg'); }");
        assert!(
            diags.is_empty(),
            "typeof check with TypeError should not be flagged"
        );
    }

    #[test]
    fn test_flags_instanceof_with_error() {
        let diags = lint("if (x instanceof Foo) { throw new Error('msg'); }");
        assert_eq!(
            diags.len(),
            1,
            "instanceof check with Error should be flagged"
        );
    }

    #[test]
    fn test_allows_non_type_check() {
        let diags = lint("if (x > 0) { throw new Error('msg'); }");
        assert!(diags.is_empty(), "non-type-check should not be flagged");
    }
}
