//! Rule: `erasing-op` (OXC)
//!
//! Detect operations that always produce a known constant regardless of the
//! other operand, such as `x * 0`, `x & 0`, `x % 1`, or `x ** 0`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags operations that erase the value of one operand.
#[derive(Debug)]
pub struct ErasingOp;

impl NativeRule for ErasingOp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "erasing-op".to_owned(),
            description: "Detect operations that always produce a known constant".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        let msg = match expr.operator {
            // x * 0 or 0 * x → always 0
            BinaryOperator::Multiplication => (is_integer_literal(&expr.left, 0)
                || is_integer_literal(&expr.right, 0))
            .then_some("Multiplying by 0 always produces 0"),
            // x & 0 or 0 & x → always 0
            BinaryOperator::BitwiseAnd => (is_integer_literal(&expr.left, 0)
                || is_integer_literal(&expr.right, 0))
            .then_some("Bitwise AND with 0 always produces 0"),
            // x % 1 → always 0
            BinaryOperator::Remainder => {
                is_integer_literal(&expr.right, 1).then_some("Remainder by 1 always produces 0")
            }
            // x ** 0 → always 1
            BinaryOperator::Exponential => is_integer_literal(&expr.right, 0)
                .then_some("Exponentiation by 0 always produces 1"),
            _ => None,
        };

        if let Some(message) = msg {
            ctx.report_warning(
                "erasing-op",
                message,
                Span::new(expr.span.start, expr.span.end),
            );
        }
    }
}

/// Check if an expression is a numeric literal with the given integer value.
///
/// Uses the raw string representation to avoid float comparison issues.
fn is_integer_literal(expr: &Expression<'_>, value: u64) -> bool {
    match expr {
        Expression::NumericLiteral(n) => {
            let expected = format!("{value}");
            n.raw.as_ref().is_some_and(|r| r.as_str() == expected)
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ErasingOp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_multiply_by_zero() {
        let diags = lint("var n = x * 0;");
        assert_eq!(diags.len(), 1, "x * 0 should be flagged");
    }

    #[test]
    fn test_flags_zero_multiply() {
        let diags = lint("var n = 0 * x;");
        assert_eq!(diags.len(), 1, "0 * x should be flagged");
    }

    #[test]
    fn test_flags_bitwise_and_zero() {
        let diags = lint("var n = x & 0;");
        assert_eq!(diags.len(), 1, "x & 0 should be flagged");
    }

    #[test]
    fn test_flags_remainder_by_one() {
        let diags = lint("var n = x % 1;");
        assert_eq!(diags.len(), 1, "x % 1 should be flagged");
    }

    #[test]
    fn test_flags_exponent_zero() {
        let diags = lint("var n = x ** 0;");
        assert_eq!(diags.len(), 1, "x ** 0 should be flagged");
    }

    #[test]
    fn test_allows_normal_multiplication() {
        let diags = lint("var n = x * 2;");
        assert!(
            diags.is_empty(),
            "normal multiplication should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_remainder() {
        let diags = lint("var n = x % 3;");
        assert!(diags.is_empty(), "normal remainder should not be flagged");
    }
}
