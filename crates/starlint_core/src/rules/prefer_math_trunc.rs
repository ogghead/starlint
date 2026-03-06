//! Rule: `prefer-math-trunc`
//!
//! Prefer `Math.trunc(x)` over bitwise hacks for integer truncation.
//! Flags `x | 0`, `x >> 0`, and `~~x`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression, AstType::UnaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // ~~x
            AstKind::UnaryExpression(outer) if outer.operator == UnaryOperator::BitwiseNot => {
                if let Expression::UnaryExpression(inner) = &outer.argument {
                    if inner.operator == UnaryOperator::BitwiseNot {
                        let source = ctx.source_text();
                        let arg_start = usize::try_from(inner.argument.span().start).unwrap_or(0);
                        let arg_end = usize::try_from(inner.argument.span().end).unwrap_or(0);
                        let arg_text = source.get(arg_start..arg_end).unwrap_or("x");

                        ctx.report(Diagnostic {
                            rule_name: "prefer-math-trunc".to_owned(),
                            message: "Use `Math.trunc(x)` instead of `~~x`".to_owned(),
                            span: Span::new(outer.span.start, outer.span.end),
                            severity: Severity::Warning,
                            help: Some("Replace with `Math.trunc()`".to_owned()),
                            fix: Some(Fix {
                                message: "Replace with `Math.trunc()`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(outer.span.start, outer.span.end),
                                    replacement: format!("Math.trunc({arg_text})"),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
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

                    let source = ctx.source_text();
                    let left_start = usize::try_from(expr.left.span().start).unwrap_or(0);
                    let left_end = usize::try_from(expr.left.span().end).unwrap_or(0);
                    let left_text = source.get(left_start..left_end).unwrap_or("x");

                    ctx.report(Diagnostic {
                        rule_name: "prefer-math-trunc".to_owned(),
                        message: format!("Use `Math.trunc(x)` instead of `x {op} 0`"),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `Math.trunc()`".to_owned()),
                        fix: Some(Fix {
                            message: "Replace with `Math.trunc()`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement: format!("Math.trunc({left_text})"),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
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
