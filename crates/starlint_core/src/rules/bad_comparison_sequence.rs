//! Rule: `bad-comparison-sequence` (OXC)
//!
//! Catch chained comparisons like `a < b < c` which don't work as expected in
//! JavaScript. In `a < b < c`, `a < b` evaluates to a boolean, which is then
//! compared to `c` — almost never the intended behavior.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags chained comparison sequences like `a < b < c`.
#[derive(Debug)]
pub struct BadComparisonSequence;

impl LintRule for BadComparisonSequence {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-comparison-sequence".to_owned(),
            description: "Catch chained comparison sequences like `a < b < c`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        // Only check comparison operators (not equality)
        if !expr.operator.is_compare() {
            return;
        }

        // Check if the left operand is also a comparison — that makes it a chain
        let Some(AstNode::BinaryExpression(left)) = ctx.node(expr.left) else {
            return;
        };

        if !left.operator.is_compare() {
            return;
        }

        // Fix: a < b < c → a < b && b < c
        #[allow(clippy::as_conversions)]
        let fix = (|| {
            let source = ctx.source_text();
            let a_span = ctx.node(left.left)?.span();
            let b_span = ctx.node(left.right)?.span();
            let c_span = ctx.node(expr.right)?.span();
            let a_text = source.get(a_span.start as usize..a_span.end as usize)?;
            let b_text = source.get(b_span.start as usize..b_span.end as usize)?;
            let c_text = source.get(c_span.start as usize..c_span.end as usize)?;
            let left_op = operator_str(left.operator);
            let right_op = operator_str(expr.operator);
            let replacement =
                format!("{a_text} {left_op} {b_text} && {b_text} {right_op} {c_text}");
            Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
                    replacement,
                }],
                is_snippet: false,
            })
        })();

        ctx.report(Diagnostic {
            rule_name: "bad-comparison-sequence".to_owned(),
            message: "Chained comparisons like `a < b < c` do not work as expected in JavaScript — \
                     the left comparison returns a boolean, which is then compared to the right operand".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

/// Convert a comparison operator to its source string.
const fn operator_str(op: BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::LessThan => "<",
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::LessEqualThan => "<=",
        BinaryOperator::GreaterEqualThan => ">=",
        _ => "??",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(BadComparisonSequence)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_chained_less_than() {
        let diags = lint("if (a < b < c) {}");
        assert_eq!(diags.len(), 1, "a < b < c should be flagged");
    }

    #[test]
    fn test_flags_chained_greater_than() {
        let diags = lint("if (a > b > c) {}");
        assert_eq!(diags.len(), 1, "a > b > c should be flagged");
    }

    #[test]
    fn test_flags_mixed_chain() {
        let diags = lint("if (a < b >= c) {}");
        assert_eq!(diags.len(), 1, "a < b >= c should be flagged");
    }

    #[test]
    fn test_allows_simple_comparison() {
        let diags = lint("if (a < b) {}");
        assert!(diags.is_empty(), "simple comparison should not be flagged");
    }

    #[test]
    fn test_allows_logical_and_comparisons() {
        let diags = lint("if (a < b && b < c) {}");
        assert!(
            diags.is_empty(),
            "proper range check with && should not be flagged"
        );
    }
}
