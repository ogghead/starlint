//! Rule: `bad-min-max-func` (OXC)
//!
//! Detect nested `Math.min(Math.max(...))` or `Math.max(Math.min(...))`
//! where the bounds are inverted, making the clamping logic incorrect.
//! For example, `Math.min(Math.max(x, 10), 5)` where min bound > max bound.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::{AstNode, CallExpressionNode};
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags nested `Math.min`/`Math.max` with inverted bounds.
#[derive(Debug)]
pub struct BadMinMaxFunc;

impl LintRule for BadMinMaxFunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-min-max-func".to_owned(),
            description: "Detect nested Math.min/Math.max with inverted bounds".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::too_many_lines)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let outer_fn = get_math_func_name(call.callee, ctx);
        let Some(outer_name) = outer_fn else {
            return;
        };

        // Look for the inner Math.min/Math.max call
        // Pattern: Math.min(Math.max(x, low), high) or Math.max(Math.min(x, high), low)
        for arg in &call.arguments {
            let Some(AstNode::CallExpression(inner_call)) = ctx.node(*arg) else {
                continue;
            };

            let inner_fn = get_math_func_name(inner_call.callee, ctx);
            let Some(inner_name) = inner_fn else {
                continue;
            };

            // Only flag when outer and inner are different (min wrapping max or vice versa)
            if outer_name != inner_name {
                // Try to extract numeric bounds
                let outer_bound = get_numeric_arg(call, inner_call, ctx);
                let inner_bound = get_numeric_arg_from_inner(inner_call, ctx);

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
                            let inner_num_span = find_numeric_literal_span(inner_call, ctx);
                            let outer_num_span =
                                find_numeric_literal_span_excluding(call, inner_call, ctx);
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
                                            let call_span =
                                                Span::new(call.span.start, call.span.end);
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
                                                    kind: FixKind::SafeFix,
                                                    message: format!("Replace with `{result}`"),
                                                    edits: vec![Edit {
                                                        span: Span::new(
                                                            call_span.start,
                                                            call_span.end,
                                                        ),
                                                        replacement: result,
                                                    }],
                                                    is_snippet: false,
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
fn get_math_func_name(callee: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(callee) else {
        return None;
    };
    let name = member.property.as_str();
    let is_math = matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Math");
    ((name == "min" || name == "max") && is_math).then(|| name.to_owned())
}

/// Get the numeric literal argument from the outer call that is NOT the inner call.
fn get_numeric_arg(
    outer: &CallExpressionNode,
    inner: &CallExpressionNode,
    ctx: &LintContext<'_>,
) -> Option<f64> {
    for arg in &outer.arguments {
        let Some(arg_node) = ctx.node(*arg) else {
            continue;
        };
        // Skip the inner call expression
        if let AstNode::CallExpression(c) = arg_node {
            if c.span == inner.span {
                continue;
            }
        }
        if let AstNode::NumericLiteral(n) = arg_node {
            return Some(n.value);
        }
    }
    None
}

/// Get the numeric literal argument from an inner Math.min/max call.
fn get_numeric_arg_from_inner(inner: &CallExpressionNode, ctx: &LintContext<'_>) -> Option<f64> {
    for arg in &inner.arguments {
        let Some(arg_node) = ctx.node(*arg) else {
            continue;
        };
        if let AstNode::NumericLiteral(n) = arg_node {
            return Some(n.value);
        }
    }
    None
}

/// Find the span of the numeric literal argument in a call.
fn find_numeric_literal_span(call: &CallExpressionNode, ctx: &LintContext<'_>) -> Option<Span> {
    for arg in &call.arguments {
        if let Some(AstNode::NumericLiteral(n)) = ctx.node(*arg) {
            return Some(Span::new(n.span.start, n.span.end));
        }
    }
    None
}

/// Find the span of the numeric literal in the outer call, excluding the inner call's args.
fn find_numeric_literal_span_excluding(
    outer: &CallExpressionNode,
    inner: &CallExpressionNode,
    ctx: &LintContext<'_>,
) -> Option<Span> {
    for arg in &outer.arguments {
        let Some(arg_node) = ctx.node(*arg) else {
            continue;
        };
        if let AstNode::CallExpression(c) = arg_node {
            if c.span == inner.span {
                continue;
            }
        }
        if let AstNode::NumericLiteral(n) = arg_node {
            return Some(Span::new(n.span.start, n.span.end));
        }
    }
    None
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(BadMinMaxFunc);

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
