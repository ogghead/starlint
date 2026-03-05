//! Rule: `prefer-type-error` (unicorn)
//!
//! Prefer throwing `TypeError` in type-checking `if` statements.
//! When checking the type of a value, throwing a `TypeError` is more
//! appropriate than a generic `Error`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, Statement, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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
            fix_kind: FixKind::SafeFix,
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
        if let Some(error_id_span) = find_error_callee_span(&if_stmt.consequent) {
            ctx.report(Diagnostic {
                rule_name: "prefer-type-error".to_owned(),
                message: "Use `new TypeError()` instead of `new Error()` for type checks"
                    .to_owned(),
                span: Span::new(if_stmt.span.start, if_stmt.span.end),
                severity: Severity::Warning,
                help: Some("Replace `Error` with `TypeError`".to_owned()),
                fix: Some(Fix {
                    message: "Replace `Error` with `TypeError`".to_owned(),
                    edits: vec![Edit {
                        span: error_id_span,
                        replacement: "TypeError".to_owned(),
                    }],
                }),
                labels: vec![],
            });
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

/// Find the span of the `Error` identifier in a `throw new Error(...)` statement.
/// Returns `None` if the statement doesn't throw a generic `Error`.
fn find_error_callee_span(stmt: &Statement<'_>) -> Option<Span> {
    match stmt {
        Statement::BlockStatement(block) => {
            if block.body.len() == 1 {
                block.body.first().and_then(find_error_callee_span)
            } else {
                None
            }
        }
        Statement::ThrowStatement(throw) => {
            if let Expression::NewExpression(new_expr) = &throw.argument {
                if let Expression::Identifier(id) = &new_expr.callee {
                    if id.name == "Error" {
                        return Some(Span::new(id.span.start, id.span.end));
                    }
                }
            }
            None
        }
        _ => None,
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
