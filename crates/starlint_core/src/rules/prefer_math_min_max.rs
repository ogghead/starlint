//! Rule: `prefer-math-min-max`
//!
//! Prefer `Math.min()` / `Math.max()` over ternary expressions for
//! clamping. A conditional like `a > b ? a : b` is equivalent to
//! `Math.max(a, b)` and the built-in call is clearer and less error-prone.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

/// Flags ternary expressions that can be replaced with `Math.min()` / `Math.max()`.
#[derive(Debug)]
pub struct PreferMathMinMax;

impl LintRule for PreferMathMinMax {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-math-min-max".to_owned(),
            description: "Prefer Math.min()/Math.max() over ternary expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ConditionalExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ConditionalExpression(cond) = node else {
            return;
        };

        // The test must be a binary comparison expression
        let Some(AstNode::BinaryExpression(binary)) = ctx.node(cond.test) else {
            return;
        };

        let source = ctx.source_text();

        // Only care about comparison operators
        let suggestion = match binary.operator {
            BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan => classify_greater(
                ctx,
                binary.left,
                binary.right,
                cond.consequent,
                cond.alternate,
                source,
            ),
            BinaryOperator::LessThan | BinaryOperator::LessEqualThan => classify_less(
                ctx,
                binary.left,
                binary.right,
                cond.consequent,
                cond.alternate,
                source,
            ),
            _ => None,
        };

        if let Some((func_name, a_text, b_text)) = suggestion {
            let replacement = format!("{func_name}({a_text}, {b_text})");
            ctx.report(Diagnostic {
                rule_name: "prefer-math-min-max".to_owned(),
                message: format!("Use `{func_name}()` instead of a ternary expression"),
                span: Span::new(cond.span.start, cond.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(cond.span.start, cond.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Extract a slice of source text for a given node ID.
fn span_text_by_id<'s>(ctx: &LintContext<'_>, id: NodeId, source: &'s str) -> Option<&'s str> {
    let node = ctx.node(id)?;
    let sp = node.span();
    let start = usize::try_from(sp.start).ok()?;
    let end = usize::try_from(sp.end).ok()?;
    source.get(start..end)
}

/// For `a > b` (or `a >= b`):
///   consequent=a, alternate=b  =>  Math.max
///   consequent=b, alternate=a  =>  Math.min
fn classify_greater<'s>(
    ctx: &LintContext<'_>,
    left: NodeId,
    right: NodeId,
    consequent: NodeId,
    alternate: NodeId,
    source: &'s str,
) -> Option<(&'static str, &'s str, &'s str)> {
    let left_text = span_text_by_id(ctx, left, source)?;
    let right_text = span_text_by_id(ctx, right, source)?;
    let cons_text = span_text_by_id(ctx, consequent, source)?;
    let alt_text = span_text_by_id(ctx, alternate, source)?;

    if left_text == cons_text && right_text == alt_text {
        Some(("Math.max", left_text, right_text))
    } else if left_text == alt_text && right_text == cons_text {
        Some(("Math.min", left_text, right_text))
    } else {
        None
    }
}

/// For `a < b` (or `a <= b`):
///   consequent=a, alternate=b  =>  Math.min
///   consequent=b, alternate=a  =>  Math.max
fn classify_less<'s>(
    ctx: &LintContext<'_>,
    left: NodeId,
    right: NodeId,
    consequent: NodeId,
    alternate: NodeId,
    source: &'s str,
) -> Option<(&'static str, &'s str, &'s str)> {
    let left_text = span_text_by_id(ctx, left, source)?;
    let right_text = span_text_by_id(ctx, right, source)?;
    let cons_text = span_text_by_id(ctx, consequent, source)?;
    let alt_text = span_text_by_id(ctx, alternate, source)?;

    if left_text == cons_text && right_text == alt_text {
        Some(("Math.min", left_text, right_text))
    } else if left_text == alt_text && right_text == cons_text {
        Some(("Math.max", left_text, right_text))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferMathMinMax)];
        lint_source(source, "test.js", &rules)
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
