//! Rule: `no-eq-null`
//!
//! Disallow `null` comparisons without type-checking operators.
//! `x == null` should use `x === null` or `x === undefined` instead.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags loose equality comparisons with `null`.
#[derive(Debug)]
pub struct NoEqNull;

impl LintRule for NoEqNull {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-eq-null".to_owned(),
            description: "Disallow `null` comparisons without type-checking operators".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        // Only check loose equality (== and !=)
        if expr.operator != BinaryOperator::Equality && expr.operator != BinaryOperator::Inequality
        {
            return;
        }

        let left_null = matches!(ctx.node(expr.left), Some(AstNode::NullLiteral(_)));
        let right_null = matches!(ctx.node(expr.right), Some(AstNode::NullLiteral(_)));

        if !left_null && !right_null {
            return;
        }

        let left_span = ctx.node(expr.left).map(AstNode::span);
        let right_span = ctx.node(expr.right).map(AstNode::span);

        let source = ctx.source_text();
        let left_end = left_span.map_or(0, |s| s.end as usize);
        let right_start = right_span.map_or(0, |s| s.start as usize);
        let between = source.get(left_end..right_start).unwrap_or("");
        let op_str = if expr.operator == BinaryOperator::Equality {
            "=="
        } else {
            "!="
        };
        let replacement_op = if expr.operator == BinaryOperator::Equality {
            "==="
        } else {
            "!=="
        };

        let fix = between.find(op_str).map(|offset| {
            let op_pos = u32::try_from(left_end.saturating_add(offset)).unwrap_or(0);
            Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace `{op_str}` with `{replacement_op}`"),
                edits: vec![Edit {
                    span: Span::new(op_pos, op_pos.saturating_add(2)),
                    replacement: replacement_op.to_owned(),
                }],
                is_snippet: false,
            }
        });

        ctx.report(Diagnostic {
            rule_name: "no-eq-null".to_owned(),
            message: "Use `===` or `!==` to compare with `null`".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{op_str}` with `{replacement_op}`")),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEqNull)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_loose_equality_null() {
        let diags = lint("if (x == null) {}");
        assert_eq!(diags.len(), 1, "x == null should be flagged");
    }

    #[test]
    fn test_flags_loose_inequality_null() {
        let diags = lint("if (x != null) {}");
        assert_eq!(diags.len(), 1, "x != null should be flagged");
    }

    #[test]
    fn test_allows_strict_equality_null() {
        let diags = lint("if (x === null) {}");
        assert!(diags.is_empty(), "x === null should not be flagged");
    }

    #[test]
    fn test_allows_strict_inequality_null() {
        let diags = lint("if (x !== null) {}");
        assert!(diags.is_empty(), "x !== null should not be flagged");
    }
}
