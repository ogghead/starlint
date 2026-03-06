//! Rule: `bad-min-max-func` (OXC)
//!
//! Detect nested `Math.min(Math.max(...))` or `Math.max(Math.min(...))`
//! where the bounds are inverted, making the clamping logic incorrect.
//! For example, `Math.min(Math.max(x, 10), 5)` where min bound > max bound.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::too_many_lines)]
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
                        // Fix: swap the bounds
                        // e.g. Math.min(Math.max(val, 10), 5) → Math.min(Math.max(val, 5), 10)
                        #[allow(clippy::as_conversions)]
                        let fix = {
                            let source = ctx.source_text();
                            let inner_num_span = find_numeric_literal_span(inner_call);
                            let outer_num_span =
                                find_numeric_literal_span_excluding(call, inner_call);
                            match (inner_num_span, outer_num_span) {
                                (Some(i_span), Some(o_span)) => {
                                    let i_text =
                                        source.get(i_span.start as usize..i_span.end as usize);
                                    let o_text =
                                        source.get(o_span.start as usize..o_span.end as usize);
                                    match (i_text, o_text) {
                                        (Some(inner_t), Some(outer_t)) => {
                                            // Swap: replace inner num with outer num and vice versa
                                            // Build by replacing in the full expression
                                            let call_span = call.span();
                                            let full_text = source.get(
                                                call_span.start as usize..call_span.end as usize,
                                            );
                                            full_text.map(|text| {
                                                // We need to swap the two numeric literals
                                                // Since spans are absolute, convert to relative
                                                let base = call_span.start as usize;
                                                let i_rel_start =
                                                    (i_span.start as usize).saturating_sub(base);
                                                let i_rel_end =
                                                    (i_span.end as usize).saturating_sub(base);
                                                let o_rel_start =
                                                    (o_span.start as usize).saturating_sub(base);
                                                let o_rel_end =
                                                    (o_span.end as usize).saturating_sub(base);
                                                let mut result = text.to_owned();
                                                // Replace the later span first to preserve positions
                                                if i_rel_start > o_rel_start {
                                                    result.replace_range(
                                                        i_rel_start..i_rel_end,
                                                        outer_t,
                                                    );
                                                    result.replace_range(
                                                        o_rel_start..o_rel_end,
                                                        inner_t,
                                                    );
                                                } else {
                                                    result.replace_range(
                                                        o_rel_start..o_rel_end,
                                                        inner_t,
                                                    );
                                                    result.replace_range(
                                                        i_rel_start..i_rel_end,
                                                        outer_t,
                                                    );
                                                }
                                                Fix {
                                                    message: format!("Replace with `{result}`"),
                                                    edits: vec![Edit {
                                                        span: Span::new(
                                                            call_span.start,
                                                            call_span.end,
                                                        ),
                                                        replacement: result,
                                                    }],
                                                }
                                            })
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            }
                        };

                        ctx.report(Diagnostic {
                            rule_name: "bad-min-max-func".to_owned(),
                            message: "Nested Math.min/Math.max have inverted bounds — \
                             the clamped range is empty"
                                .to_owned(),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix,
                            labels: vec![],
                        });
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

/// Find the span of the numeric literal argument in a call.
fn find_numeric_literal_span(call: &oxc_ast::ast::CallExpression<'_>) -> Option<Span> {
    for arg in &call.arguments {
        let Some(expr) = arg.as_expression() else {
            continue;
        };
        if let Expression::NumericLiteral(n) = expr {
            return Some(Span::new(n.span.start, n.span.end));
        }
    }
    None
}

/// Find the span of the numeric literal in the outer call, excluding the inner call's args.
fn find_numeric_literal_span_excluding(
    outer: &oxc_ast::ast::CallExpression<'_>,
    inner: &oxc_ast::ast::CallExpression<'_>,
) -> Option<Span> {
    for arg in &outer.arguments {
        let Some(expr) = arg.as_expression() else {
            continue;
        };
        if let Expression::CallExpression(c) = expr {
            if c.span == inner.span {
                continue;
            }
        }
        if let Expression::NumericLiteral(n) = expr {
            return Some(Span::new(n.span.start, n.span.end));
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
