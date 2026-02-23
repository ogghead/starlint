//! Rule: `prefer-math-trunc`
//!
//! Prefer `Math.trunc(x)` over bitwise hacks for integer truncation.
//! Flags `x | 0`, `x >> 0`, and `~~x`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags bitwise truncation patterns — prefer `Math.trunc()`.
#[derive(Debug)]
pub struct PreferMathTrunc;

/// Check if a node is the numeric literal `0`.
fn is_zero(id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.node(id), Some(AstNode::NumericLiteral(lit)) if lit.value.abs() < f64::EPSILON)
}

impl LintRule for PreferMathTrunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-math-trunc".to_owned(),
            description: "Prefer `Math.trunc(x)` over bitwise truncation".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression, AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // ~~x
            AstNode::UnaryExpression(outer) if outer.operator == UnaryOperator::BitwiseNot => {
                if let Some(AstNode::UnaryExpression(inner)) = ctx.node(outer.argument) {
                    if inner.operator == UnaryOperator::BitwiseNot {
                        let Some(inner_arg_node) = ctx.node(inner.argument) else {
                            return;
                        };
                        let inner_arg_span = inner_arg_node.span();
                        let source = ctx.source_text();
                        let arg_start = usize::try_from(inner_arg_span.start).unwrap_or(0);
                        let arg_end = usize::try_from(inner_arg_span.end).unwrap_or(0);
                        let arg_text = source.get(arg_start..arg_end).unwrap_or("x");

                        ctx.report(Diagnostic {
                            rule_name: "prefer-math-trunc".to_owned(),
                            message: "Use `Math.trunc(x)` instead of `~~x`".to_owned(),
                            span: Span::new(outer.span.start, outer.span.end),
                            severity: Severity::Warning,
                            help: Some("Replace with `Math.trunc()`".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SuggestionFix,
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
            AstNode::BinaryExpression(expr) => {
                let is_truncation = match expr.operator {
                    BinaryOperator::BitwiseOR | BinaryOperator::ShiftRight => {
                        is_zero(expr.right, ctx)
                    }
                    _ => false,
                };

                if is_truncation {
                    let op = match expr.operator {
                        BinaryOperator::BitwiseOR => "|",
                        BinaryOperator::ShiftRight => ">>",
                        _ => return,
                    };

                    let Some(left_node) = ctx.node(expr.left) else {
                        return;
                    };
                    let left_span = left_node.span();
                    let source = ctx.source_text();
                    let left_start = usize::try_from(left_span.start).unwrap_or(0);
                    let left_end = usize::try_from(left_span.end).unwrap_or(0);
                    let left_text = source.get(left_start..left_end).unwrap_or("x");

                    ctx.report(Diagnostic {
                        rule_name: "prefer-math-trunc".to_owned(),
                        message: format!("Use `Math.trunc(x)` instead of `x {op} 0`"),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `Math.trunc()`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
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

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferMathTrunc)];
        lint_source(source, "test.js", &rules)
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
