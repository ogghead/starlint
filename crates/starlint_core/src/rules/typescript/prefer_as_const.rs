//! Rule: `typescript/prefer-as-const`
//!
//! Prefer `as const` over literal type assertion. When a value is asserted to
//! its own literal type (e.g. `"hello" as "hello"` or `1 as 1`), `as const`
//! is clearer and prevents the assertion from drifting out of sync with the
//! value.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, TSLiteral, TSType};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags literal type assertions that could use `as const` instead.
#[derive(Debug)]
pub struct PreferAsConst;

impl NativeRule for PreferAsConst {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-as-const".to_owned(),
            description: "Prefer `as const` over literal type assertion".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::TSAsExpression(expr) => {
                if is_literal_self_assertion(&expr.expression, &expr.type_annotation) {
                    ctx.report_warning(
                        "typescript/prefer-as-const",
                        "Use `as const` instead of asserting a literal to its own type",
                        Span::new(expr.span.start, expr.span.end),
                    );
                }
            }
            AstKind::TSTypeAssertion(expr) => {
                if is_literal_self_assertion(&expr.expression, &expr.type_annotation) {
                    ctx.report_warning(
                        "typescript/prefer-as-const",
                        "Use `as const` instead of asserting a literal to its own type",
                        Span::new(expr.span.start, expr.span.end),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Check whether an expression is a literal being asserted to its own literal type.
///
/// Returns `true` for patterns like `"hello" as "hello"` or `1 as 1` where the
/// expression value matches the type annotation's literal value.
fn is_literal_self_assertion(expression: &Expression<'_>, type_annotation: &TSType<'_>) -> bool {
    let TSType::TSLiteralType(lit_type) = type_annotation else {
        return false;
    };

    match (&lit_type.literal, expression) {
        (TSLiteral::StringLiteral(type_str), Expression::StringLiteral(expr_str)) => {
            type_str.value == expr_str.value
        }
        (TSLiteral::NumericLiteral(type_num), Expression::NumericLiteral(expr_num)) => {
            // Compare raw source representations to handle edge cases like -0 vs 0.
            // Fall back to value comparison when raw is unavailable.
            match (&type_num.raw, &expr_num.raw) {
                (Some(type_raw), Some(expr_raw)) => type_raw == expr_raw,
                _ => (type_num.value - expr_num.value).abs() < f64::EPSILON,
            }
        }
        (TSLiteral::BooleanLiteral(type_bool), Expression::BooleanLiteral(expr_bool)) => {
            type_bool.value == expr_bool.value
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferAsConst)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_literal_self_assertion() {
        let diags = lint(r#"let x = "hello" as "hello";"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal asserted to its own type should be flagged"
        );
    }

    #[test]
    fn test_flags_numeric_literal_self_assertion() {
        let diags = lint("let x = 1 as 1;");
        assert_eq!(
            diags.len(),
            1,
            "numeric literal asserted to its own type should be flagged"
        );
    }

    #[test]
    fn test_allows_as_const() {
        let diags = lint(r#"let x = "hello" as const;"#);
        assert!(diags.is_empty(), "`as const` should not be flagged");
    }

    #[test]
    fn test_allows_different_type_assertion() {
        let diags = lint("let x = y as string;");
        assert!(
            diags.is_empty(),
            "assertion to a different type should not be flagged"
        );
    }
}
