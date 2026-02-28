//! Rule: `bad-min-max-func` (OXC)
//!
//! Detect nested `Math.min(Math.max(...))` or `Math.max(Math.min(...))`
//! where the bounds are inverted, making the clamping logic incorrect.
//! For example, `Math.min(Math.max(x, 10), 5)` where min bound > max bound.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags nested `Math.min`/`Math.max` with inverted bounds.
#[derive(Debug)]
pub struct BadMinMaxFunc;

impl NativeRule for BadMinMaxFunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-min-max-func".to_owned(),
            description: "Detect nested Math.min/Math.max with inverted bounds".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let outer_fn = get_math_func_name(&call.callee);
        let Some(outer_name) = outer_fn else {
            return;
        };

        // Look for the inner Math.min/Math.max call
        // Pattern: Math.min(Math.max(x, low), high) or Math.max(Math.min(x, high), low)
        for arg in &call.arguments {
            let Some(expr) = arg.as_expression() else {
                continue;
            };
            let Expression::CallExpression(inner_call) = expr else {
                continue;
            };

            let inner_fn = get_math_func_name(&inner_call.callee);
            let Some(inner_name) = inner_fn else {
                continue;
            };

            // Only flag when outer and inner are different (min wrapping max or vice versa)
            if outer_name != inner_name {
                // Try to extract numeric bounds
                let outer_bound = get_numeric_arg(call, inner_call);
                let inner_bound = get_numeric_arg_from_inner(inner_call);

                if let (Some(outer_val), Some(inner_val)) = (outer_bound, inner_bound) {
                    // Math.min(Math.max(x, low), high): low should be < high
                    // Math.max(Math.min(x, high), low): high should be > low
                    let inverted = if outer_name == "min" {
                        // outer is min(_, high), inner is max(_, low)
                        // inverted if low > high
                        inner_val > outer_val
                    } else {
                        // outer is max(_, low), inner is min(_, high)
                        // inverted if high < low
                        inner_val < outer_val
                    };

                    if inverted {
                        ctx.report_warning(
                            "bad-min-max-func",
                            "Nested Math.min/Math.max have inverted bounds — \
                             the clamped range is empty",
                            Span::new(call.span.start, call.span.end),
                        );
                    }
                }
            }
        }
    }
}

/// Get the Math function name if the callee is `Math.min` or `Math.max`.
fn get_math_func_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee {
        Expression::StaticMemberExpression(member) => {
            let name = member.property.name.as_str();
            ((name == "min" || name == "max")
                && matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Math"))
            .then_some(name)
        }
        _ => None,
    }
}

/// Get the numeric literal argument from the outer call that is NOT the inner call.
fn get_numeric_arg(
    outer: &oxc_ast::ast::CallExpression<'_>,
    inner: &oxc_ast::ast::CallExpression<'_>,
) -> Option<f64> {
    for arg in &outer.arguments {
        let Some(expr) = arg.as_expression() else {
            continue;
        };
        // Skip the inner call expression
        if let Expression::CallExpression(c) = expr {
            if c.span == inner.span {
                continue;
            }
        }
        if let Expression::NumericLiteral(n) = expr {
            return Some(n.value);
        }
    }
    None
}

/// Get the numeric literal argument from an inner Math.min/max call.
fn get_numeric_arg_from_inner(inner: &oxc_ast::ast::CallExpression<'_>) -> Option<f64> {
    for arg in &inner.arguments {
        let Some(expr) = arg.as_expression() else {
            continue;
        };
        if let Expression::NumericLiteral(n) = expr {
            return Some(n.value);
        }
    }
    None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BadMinMaxFunc)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_inverted_min_max() {
        // min bound (10) > max bound (5) → inverted
        let diags = lint("var x = Math.min(Math.max(val, 10), 5);");
        assert_eq!(
            diags.len(),
            1,
            "inverted Math.min/Math.max bounds should be flagged"
        );
    }

    #[test]
    fn test_flags_inverted_max_min() {
        // Math.max(Math.min(val, 5), 10) → inner bound 5 < outer bound 10 → inverted
        let diags = lint("var x = Math.max(Math.min(val, 5), 10);");
        assert_eq!(
            diags.len(),
            1,
            "inverted Math.max/Math.min bounds should be flagged"
        );
    }

    #[test]
    fn test_allows_correct_clamp() {
        // min bound (0) < max bound (10) → correct
        let diags = lint("var x = Math.min(Math.max(val, 0), 10);");
        assert!(
            diags.is_empty(),
            "correct Math.min/Math.max bounds should not be flagged"
        );
    }

    #[test]
    fn test_allows_simple_min() {
        let diags = lint("var x = Math.min(a, b);");
        assert!(diags.is_empty(), "simple Math.min should not be flagged");
    }
}
