//! Rule: `prefer-math-trunc`
//!
//! Prefer `Math.trunc(x)` over bitwise hacks for integer truncation.
//! Flags `x | 0`, `x >> 0`, and `~~x`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags bitwise truncation patterns — prefer `Math.trunc()`.
#[derive(Debug)]
pub struct PreferMathTrunc;

/// Check if an expression is the numeric literal `0`.
fn is_zero(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::NumericLiteral(lit) if lit.value.abs() < f64::EPSILON)
}

impl NativeRule for PreferMathTrunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-math-trunc".to_owned(),
            description: "Prefer `Math.trunc(x)` over bitwise truncation".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // ~~x
            AstKind::UnaryExpression(outer) if outer.operator == UnaryOperator::BitwiseNot => {
                if let Expression::UnaryExpression(inner) = &outer.argument {
                    if inner.operator == UnaryOperator::BitwiseNot {
                        ctx.report_warning(
                            "prefer-math-trunc",
                            "Use `Math.trunc(x)` instead of `~~x`",
                            Span::new(outer.span.start, outer.span.end),
                        );
                    }
                }
            }
            // x | 0 or x >> 0
            AstKind::BinaryExpression(expr) => {
                let is_truncation = match expr.operator {
                    BinaryOperator::BitwiseOR | BinaryOperator::ShiftRight => is_zero(&expr.right),
                    _ => false,
                };

                if is_truncation {
                    let op = match expr.operator {
                        BinaryOperator::BitwiseOR => "|",
                        BinaryOperator::ShiftRight => ">>",
                        _ => return,
                    };
                    ctx.report_warning(
                        "prefer-math-trunc",
                        &format!("Use `Math.trunc(x)` instead of `x {op} 0`"),
                        Span::new(expr.span.start, expr.span.end),
                    );
                }
            }
            _ => {}
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferMathTrunc)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_double_bitwise_not() {
        let diags = lint("const n = ~~x;");
        assert_eq!(diags.len(), 1, "should flag ~~x");
    }

    #[test]
    fn test_flags_bitwise_or_zero() {
        let diags = lint("const n = x | 0;");
        assert_eq!(diags.len(), 1, "should flag x | 0");
    }

    #[test]
    fn test_flags_shift_right_zero() {
        let diags = lint("const n = x >> 0;");
        assert_eq!(diags.len(), 1, "should flag x >> 0");
    }

    #[test]
    fn test_allows_math_trunc() {
        let diags = lint("const n = Math.trunc(x);");
        assert!(diags.is_empty(), "Math.trunc() should not be flagged");
    }
}
