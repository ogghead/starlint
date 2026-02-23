//! Rule: `no-self-compare`
//!
//! Disallow comparisons where both sides are exactly the same.
//! Comparing a value against itself is almost always a bug. The only
//! valid use case (`x !== x` to check for `NaN`) should use `Number.isNaN()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags comparisons where both operands are the same identifier.
#[derive(Debug)]
pub struct NoSelfCompare;

impl LintRule for NoSelfCompare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-self-compare".to_owned(),
            description: "Disallow comparisons where both sides are the same".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        // Compare source text of both sides to detect identical expressions.
        let Some(left_node) = ctx.node(expr.left) else {
            return;
        };
        let Some(right_node) = ctx.node(expr.right) else {
            return;
        };
        let left_ast_span = left_node.span();
        let right_ast_span = right_node.span();

        let left_start = usize::try_from(left_ast_span.start).unwrap_or(0);
        let left_end = usize::try_from(left_ast_span.end).unwrap_or(0);
        let right_start = usize::try_from(right_ast_span.start).unwrap_or(0);
        let right_end = usize::try_from(right_ast_span.end).unwrap_or(0);

        let source = ctx.source_text();
        let left_text = source.get(left_start..left_end);
        let right_text = source.get(right_start..right_end);

        if let (Some(left), Some(right)) = (left_text, right_text) {
            if !left.is_empty() && left == right {
                // For `x !== x`, offer fix to `Number.isNaN(x)`
                let fix = matches!(expr.operator, BinaryOperator::StrictInequality).then(|| {
                    let replacement = format!("Number.isNaN({left})");
                    Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr.span.start, expr.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: "no-self-compare".to_owned(),
                    message: format!("Comparing `{left}` against itself is always predictable"),
                    span: Span::new(expr.span.start, expr.span.end),
                    severity: Severity::Error,
                    help: Some("If testing for NaN, use `Number.isNaN(value)` instead".to_owned()),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoSelfCompare)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_self_strict_equality() {
        let diags = lint("if (x === x) {}");
        assert_eq!(diags.len(), 1, "x === x should be flagged");
    }

    #[test]
    fn test_flags_self_inequality() {
        let diags = lint("if (x !== x) {}");
        assert_eq!(
            diags.len(),
            1,
            "x !== x should be flagged (use Number.isNaN)"
        );
    }

    #[test]
    fn test_flags_self_less_than() {
        let diags = lint("if (x < x) {}");
        assert_eq!(diags.len(), 1, "x < x should be flagged");
    }

    #[test]
    fn test_allows_different_operands() {
        let diags = lint("if (x === y) {}");
        assert!(diags.is_empty(), "different operands should not be flagged");
    }

    #[test]
    fn test_allows_arithmetic() {
        let diags = lint("const y = x + x;");
        assert!(diags.is_empty(), "arithmetic is not a comparison");
    }
}
