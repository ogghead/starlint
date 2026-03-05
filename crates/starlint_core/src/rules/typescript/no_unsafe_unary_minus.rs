//! Rule: `typescript/no-unsafe-unary-minus`
//!
//! Disallow unary minus on non-numeric types. Applying the unary minus
//! operator to a non-numeric value produces `NaN`, which is almost always
//! a bug. This rule flags obvious cases like `-"string"`, `-true`, `-false`,
//! `-null`, `-undefined`, `-{}`, and `-[]`.
//!
//! Since we cannot perform full type checking, only literal expressions
//! and well-known non-numeric identifiers are detected.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unary minus applied to obviously non-numeric values.
#[derive(Debug)]
pub struct NoUnsafeUnaryMinus;

impl NativeRule for NoUnsafeUnaryMinus {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-unary-minus".to_owned(),
            description: "Disallow unary minus on non-numeric types".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::UnaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::UnaryExpression(expr) = kind else {
            return;
        };

        if expr.operator != UnaryOperator::UnaryNegation {
            return;
        }

        if let Some(description) = is_non_numeric_operand(&expr.argument) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-unary-minus".to_owned(),
                message: format!("Unary minus on {description} produces `NaN`"),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check whether an expression is obviously non-numeric.
///
/// Returns a human-readable description of the non-numeric type when the
/// operand is clearly not a number, or `None` when it could be numeric.
fn is_non_numeric_operand(expr: &Expression<'_>) -> Option<&'static str> {
    match expr {
        Expression::StringLiteral(_) => Some("a string literal"),
        Expression::BooleanLiteral(_) => Some("a boolean literal"),
        Expression::NullLiteral(_) => Some("`null`"),
        Expression::ObjectExpression(_) => Some("an object literal"),
        Expression::ArrayExpression(_) => Some("an array literal"),
        Expression::Identifier(ident) if ident.name == "undefined" => Some("`undefined`"),
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeUnaryMinus)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_negate_string() {
        let diags = lint(r#"let x = -"hello";"#);
        assert_eq!(
            diags.len(),
            1,
            "negating a string literal should be flagged"
        );
    }

    #[test]
    fn test_flags_negate_boolean() {
        let diags = lint("let x = -true;");
        assert_eq!(
            diags.len(),
            1,
            "negating a boolean literal should be flagged"
        );
    }

    #[test]
    fn test_flags_negate_null() {
        let diags = lint("let x = -null;");
        assert_eq!(diags.len(), 1, "negating null should be flagged");
    }

    #[test]
    fn test_flags_negate_object() {
        let diags = lint("let x = -{};");
        assert_eq!(
            diags.len(),
            1,
            "negating an object literal should be flagged"
        );
    }

    #[test]
    fn test_allows_negate_number() {
        let diags = lint("let x = -42;");
        assert!(
            diags.is_empty(),
            "negating a number literal should not be flagged"
        );
    }
}
