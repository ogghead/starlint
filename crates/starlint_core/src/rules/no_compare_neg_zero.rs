//! Rule: `no-compare-neg-zero`
//!
//! Disallow comparing against `-0`. The expression `x === -0` does not
//! work as expected because `-0 === 0` is `true` in JavaScript.
//! Use `Object.is(x, -0)` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags comparisons against `-0`.
#[derive(Debug)]
pub struct NoCompareNegZero;

impl NativeRule for NoCompareNegZero {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-compare-neg-zero".to_owned(),
            description: "Disallow comparing against `-0`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        // Only flag comparison operators
        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        let has_neg_zero = is_negative_zero(&expr.left) || is_negative_zero(&expr.right);

        if has_neg_zero {
            // Fix: `x === -0` → `Object.is(x, -0)`
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let (value_expr, _neg_zero_expr) = if is_negative_zero(&expr.right) {
                    (&expr.left, &expr.right)
                } else {
                    (&expr.right, &expr.left)
                };
                let val_span = value_expr.span();
                source
                    .get(val_span.start as usize..val_span.end as usize)
                    .map(|val_text| {
                        let replacement = format!("Object.is({val_text}, -0)");
                        Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        }
                    })
            };

            ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                rule_name: "no-compare-neg-zero".to_owned(),
                message: format!(
                    "Do not use the `{}` operator to compare against `-0`",
                    expr.operator.as_str()
                ),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: Some("Use `Object.is(x, -0)` to test for negative zero".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is the literal `-0`.
fn is_negative_zero(expr: &Expression<'_>) -> bool {
    if let Expression::UnaryExpression(unary) = expr {
        if unary.operator == UnaryOperator::UnaryNegation {
            if let Expression::NumericLiteral(lit) = &unary.argument {
                return lit.value == 0.0;
            }
        }
    }
    false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCompareNegZero)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_strict_equality_neg_zero() {
        let diags = lint("if (x === -0) {}");
        assert_eq!(diags.len(), 1, "=== -0 should be flagged");
    }

    #[test]
    fn test_flags_loose_equality_neg_zero() {
        let diags = lint("if (x == -0) {}");
        assert_eq!(diags.len(), 1, "== -0 should be flagged");
    }

    #[test]
    fn test_flags_inequality_neg_zero() {
        let diags = lint("if (x !== -0) {}");
        assert_eq!(diags.len(), 1, "!== -0 should be flagged");
    }

    #[test]
    fn test_flags_less_than_neg_zero() {
        let diags = lint("if (x < -0) {}");
        assert_eq!(diags.len(), 1, "< -0 should be flagged");
    }

    #[test]
    fn test_flags_neg_zero_on_left() {
        let diags = lint("if (-0 === x) {}");
        assert_eq!(diags.len(), 1, "-0 on left side should be flagged");
    }

    #[test]
    fn test_allows_comparison_with_zero() {
        let diags = lint("if (x === 0) {}");
        assert!(diags.is_empty(), "comparison with 0 should not be flagged");
    }

    #[test]
    fn test_allows_comparison_with_number() {
        let diags = lint("if (x === -1) {}");
        assert!(diags.is_empty(), "comparison with -1 should not be flagged");
    }

    #[test]
    fn test_allows_arithmetic() {
        let diags = lint("const y = x + -0;");
        assert!(diags.is_empty(), "arithmetic with -0 should not be flagged");
    }
}
