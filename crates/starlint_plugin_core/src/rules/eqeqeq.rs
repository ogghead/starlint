//! Rule: `eqeqeq`
//!
//! Require `===` and `!==` instead of `==` and `!=`.
//! The loose equality operators perform type coercion which is a common
//! source of bugs.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `==` and `!=` operators, suggesting `===` and `!==` instead.
#[derive(Debug)]
pub struct Eqeqeq;

impl LintRule for Eqeqeq {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "eqeqeq".to_owned(),
            description: "Require `===` and `!==`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if let AstNode::BinaryExpression(expr) = node {
            let (replacement, label) = match expr.operator {
                BinaryOperator::Equality => ("===", "=="),
                BinaryOperator::Inequality => ("!==", "!="),
                _ => return,
            };

            // Resolve left/right NodeIds to get their spans
            let left_end = ctx
                .node(expr.left)
                .map_or(expr.span.start, |n| n.span().end);
            let right_start = ctx
                .node(expr.right)
                .map_or(expr.span.end, |n| n.span().start);

            // Search only between the operands to avoid matching operators
            // inside string literals (e.g., `"a == b" == x`).
            let op_span = find_operator_span(ctx.source_text(), left_end, right_start, label);

            ctx.report(Diagnostic {
                rule_name: "eqeqeq".to_owned(),
                message: format!("Expected `{replacement}` and instead saw `{label}`"),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: Some(format!("Use `{replacement}` instead of `{label}`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace `{label}` with `{replacement}`"),
                    edits: vec![Edit {
                        span: op_span,
                        replacement: replacement.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Find the span of the operator within a binary expression.
///
/// Searches the source text between `start` and `end` for the operator string.
/// Falls back to the full expression span if not found.
fn find_operator_span(source: &str, start: u32, end: u32, operator: &str) -> Span {
    let source_len = u32::try_from(source.len()).unwrap_or(u32::MAX);
    let clamped_start = usize::try_from(start.min(source_len)).unwrap_or(0);
    let clamped_end = usize::try_from(end.min(source_len)).unwrap_or(0);

    if let Some(slice) = source.get(clamped_start..clamped_end) {
        if let Some(offset) = slice.find(operator) {
            let op_start = start.saturating_add(u32::try_from(offset).unwrap_or(0));
            let op_end = op_start.saturating_add(u32::try_from(operator.len()).unwrap_or(0));
            return Span::new(op_start, op_end);
        }
    }

    Span::new(start, end)
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(Eqeqeq)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_loose_equality() {
        let diags = lint("if (a == b) {}");
        assert_eq!(diags.len(), 1, "should flag == operator");
        let first = diags.first();
        assert!(
            first.is_some_and(|d| d.fix.is_some()),
            "should provide a fix"
        );
    }

    #[test]
    fn test_flags_loose_inequality() {
        let diags = lint("if (a != b) {}");
        assert_eq!(diags.len(), 1, "should flag != operator");
    }

    #[test]
    fn test_fix_targets_operator_not_string_content() {
        // Regression: `"a == b" == x` must fix the operator between
        // the string literal and `x`, not the `==` inside the string.
        let source = r#"if ("a == b" == x) {}"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "should flag == operator");
        if let Some(diag) = diags.first() {
            if let Some(fix) = &diag.fix {
                if let Some(edit) = fix.edits.first() {
                    let start = usize::try_from(edit.span.start).unwrap_or(0);
                    let end = usize::try_from(edit.span.end).unwrap_or(0);
                    let fixed_slice = source.get(start..end).unwrap_or("");
                    assert_eq!(
                        fixed_slice, "==",
                        "fix span should target the actual operator"
                    );
                }
            }
        }
    }

    #[test]
    fn test_allows_strict_equality() {
        let diags = lint("if (a === b && c !== d) {}");
        assert!(diags.is_empty(), "strict equality should not be flagged");
    }
}
