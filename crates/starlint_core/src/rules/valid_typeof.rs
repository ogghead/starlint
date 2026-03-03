//! Rule: `valid-typeof`
//!
//! Enforce comparing `typeof` expressions against valid type strings.
//! The `typeof` operator returns one of: "undefined", "object", "boolean",
//! "number", "string", "function", "symbol", "bigint". Any other comparison
//! is almost certainly a typo.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Valid return values from the `typeof` operator.
const VALID_TYPEOF_VALUES: &[&str] = &[
    "undefined",
    "object",
    "boolean",
    "number",
    "string",
    "function",
    "symbol",
    "bigint",
];

/// Flags `typeof` comparisons against invalid type strings.
#[derive(Debug)]
pub struct ValidTypeof;

impl NativeRule for ValidTypeof {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "valid-typeof".to_owned(),
            description: "Enforce comparing `typeof` expressions against valid strings".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        // Only check equality/inequality comparisons
        if !expr.operator.is_equality() {
            return;
        }

        // Check both orderings: typeof x === "..." and "..." === typeof x
        if is_typeof(&expr.left) {
            check_typeof_value(&expr.right, expr.span, ctx);
        } else if is_typeof(&expr.right) {
            check_typeof_value(&expr.left, expr.span, ctx);
        }
    }
}

/// Check whether an expression is a `typeof` unary expression.
fn is_typeof(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::UnaryExpression(u) if u.operator == UnaryOperator::Typeof)
}

/// If the other side of the comparison is a string literal, check it's a valid typeof value.
fn check_typeof_value(
    expr: &Expression<'_>,
    full_span: oxc_span::Span,
    ctx: &mut NativeLintContext<'_>,
) {
    let Expression::StringLiteral(lit) = expr else {
        return;
    };

    let value = lit.value.as_str();
    if !VALID_TYPEOF_VALUES.contains(&value) {
        ctx.report_error(
            "valid-typeof",
            &format!("Invalid typeof comparison value `\"{value}\"`"),
            Span::new(full_span.start, full_span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ValidTypeof)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_valid_typeof() {
        for val in VALID_TYPEOF_VALUES {
            let source = format!(r#"if (typeof x === "{val}") {{}}"#);
            let diags = lint(&source);
            assert!(diags.is_empty(), "typeof === \"{val}\" should be valid");
        }
    }

    #[test]
    fn test_flags_invalid_typeof() {
        let diags = lint(r#"if (typeof x === "strig") {}"#);
        assert_eq!(diags.len(), 1, "typo 'strig' should be flagged");
    }

    #[test]
    fn test_flags_invalid_typeof_reversed() {
        let diags = lint(r#"if ("nubmer" === typeof x) {}"#);
        assert_eq!(
            diags.len(),
            1,
            "reversed comparison with typo should be flagged"
        );
    }

    #[test]
    fn test_flags_null_typeof() {
        let diags = lint(r#"if (typeof x === "null") {}"#);
        assert_eq!(diags.len(), 1, "'null' is not a valid typeof value");
    }

    #[test]
    fn test_allows_non_equality_operator() {
        let diags = lint(r#"const x = typeof y + "strig";"#);
        assert!(
            diags.is_empty(),
            "non-equality operator should not be checked"
        );
    }

    #[test]
    fn test_allows_no_string_literal() {
        let diags = lint("if (typeof x === y) {}");
        assert!(
            diags.is_empty(),
            "comparison against variable should not be checked"
        );
    }
}
