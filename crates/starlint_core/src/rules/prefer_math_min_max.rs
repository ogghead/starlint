//! Rule: `prefer-math-min-max`
//!
//! Prefer `Math.min()` / `Math.max()` over ternary expressions for
//! clamping. A conditional like `a > b ? a : b` is equivalent to
//! `Math.max(a, b)` and the built-in call is clearer and less error-prone.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags ternary expressions that can be replaced with `Math.min()` / `Math.max()`.
#[derive(Debug)]
pub struct PreferMathMinMax;

impl NativeRule for PreferMathMinMax {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-math-min-max".to_owned(),
            description: "Prefer Math.min()/Math.max() over ternary expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ConditionalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ConditionalExpression(cond) = kind else {
            return;
        };

        // The test must be a binary comparison expression
        let Expression::BinaryExpression(binary) = &cond.test else {
            return;
        };

        // Only care about comparison operators
        let suggestion = match binary.operator {
            BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan => classify_greater(
                &binary.left,
                &binary.right,
                &cond.consequent,
                &cond.alternate,
                ctx.source_text(),
            ),
            BinaryOperator::LessThan | BinaryOperator::LessEqualThan => classify_less(
                &binary.left,
                &binary.right,
                &cond.consequent,
                &cond.alternate,
                ctx.source_text(),
            ),
            _ => None,
        };

        if let Some(func_name) = suggestion {
            ctx.report_warning(
                "prefer-math-min-max",
                &format!("Use `{func_name}()` instead of a ternary expression"),
                Span::new(cond.span.start, cond.span.end),
            );
        }
    }
}

/// Extract a slice of source text for a given expression span.
fn span_text<'s>(expr: &Expression<'_>, source: &'s str) -> Option<&'s str> {
    let sp = expr.span();
    let start = usize::try_from(sp.start).ok()?;
    let end = usize::try_from(sp.end).ok()?;
    source.get(start..end)
}

/// For `a > b` (or `a >= b`):
///   consequent=a, alternate=b  =>  Math.max
///   consequent=b, alternate=a  =>  Math.min
fn classify_greater(
    left: &Expression<'_>,
    right: &Expression<'_>,
    consequent: &Expression<'_>,
    alternate: &Expression<'_>,
    source: &str,
) -> Option<&'static str> {
    let left_text = span_text(left, source)?;
    let right_text = span_text(right, source)?;
    let cons_text = span_text(consequent, source)?;
    let alt_text = span_text(alternate, source)?;

    if left_text == cons_text && right_text == alt_text {
        Some("Math.max")
    } else if left_text == alt_text && right_text == cons_text {
        Some("Math.min")
    } else {
        None
    }
}

/// For `a < b` (or `a <= b`):
///   consequent=a, alternate=b  =>  Math.min
///   consequent=b, alternate=a  =>  Math.max
fn classify_less(
    left: &Expression<'_>,
    right: &Expression<'_>,
    consequent: &Expression<'_>,
    alternate: &Expression<'_>,
    source: &str,
) -> Option<&'static str> {
    let left_text = span_text(left, source)?;
    let right_text = span_text(right, source)?;
    let cons_text = span_text(consequent, source)?;
    let alt_text = span_text(alternate, source)?;

    if left_text == cons_text && right_text == alt_text {
        Some("Math.min")
    } else if left_text == alt_text && right_text == cons_text {
        Some("Math.max")
    } else {
        None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferMathMinMax)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_greater_than_max() {
        let diags = lint("const x = a > b ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a > b ? a : b should be flagged as Math.max"
        );
    }

    #[test]
    fn test_flags_less_than_min() {
        let diags = lint("const x = a < b ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a < b ? a : b should be flagged as Math.min"
        );
    }

    #[test]
    fn test_flags_greater_than_reversed_min() {
        let diags = lint("const x = a > b ? b : a;");
        assert_eq!(
            diags.len(),
            1,
            "a > b ? b : a should be flagged as Math.min"
        );
    }

    #[test]
    fn test_flags_greater_equal() {
        let diags = lint("const x = a >= b ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a >= b ? a : b should be flagged as Math.max"
        );
    }

    #[test]
    fn test_flags_less_equal() {
        let diags = lint("const x = a <= b ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a <= b ? a : b should be flagged as Math.min"
        );
    }

    #[test]
    fn test_allows_different_branches() {
        let diags = lint("const x = a > b ? c : d;");
        assert!(
            diags.is_empty(),
            "branches not matching comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_math_max() {
        let diags = lint("Math.max(a, b);");
        assert!(diags.is_empty(), "Math.max call should not be flagged");
    }

    #[test]
    fn test_allows_non_comparison_ternary() {
        let diags = lint("const x = a === b ? a : b;");
        assert!(diags.is_empty(), "equality test should not be flagged");
    }
}
