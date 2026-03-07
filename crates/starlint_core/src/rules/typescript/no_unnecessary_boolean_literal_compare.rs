//! Rule: `typescript/no-unnecessary-boolean-literal-compare`
//!
//! Disallow unnecessary equality comparisons against boolean literals.
//! Comparisons like `x === true` or `x === false` are redundant when `x`
//! is already a boolean. Prefer `x` or `!x` for cleaner, more idiomatic code.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unnecessary-boolean-literal-compare";

/// Flags comparisons where one side is a boolean literal and the operator
/// is `==`, `===`, `!=`, or `!==`.
#[derive(Debug)]
pub struct NoUnnecessaryBooleanLiteralCompare;

impl LintRule for NoUnnecessaryBooleanLiteralCompare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary comparisons against boolean literals".to_owned(),
            category: Category::Suggestion,
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

        // Only check equality operators
        if !matches!(
            expr.operator,
            BinaryOperator::Equality
                | BinaryOperator::StrictEquality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictInequality
        ) {
            return;
        }

        let left_is_bool = is_boolean_literal(expr.left, ctx);
        let right_is_bool = is_boolean_literal(expr.right, ctx);

        if left_is_bool || right_is_bool {
            let op_str = match expr.operator {
                BinaryOperator::Equality => "==",
                BinaryOperator::StrictEquality => "===",
                BinaryOperator::Inequality => "!=",
                BinaryOperator::StrictInequality => "!==",
                _ => return,
            };

            let bool_val = if left_is_bool {
                boolean_value(expr.left, ctx)
            } else {
                boolean_value(expr.right, ctx)
            };

            let bool_str = if bool_val.unwrap_or(true) {
                "true"
            } else {
                "false"
            };

            // Build fix: determine if we need negation
            let is_equality = matches!(
                expr.operator,
                BinaryOperator::Equality | BinaryOperator::StrictEquality
            );
            let needs_negation = if bool_val.unwrap_or(true) {
                !is_equality // `!== true` or `!= true` -> negate
            } else {
                is_equality // `=== false` or `== false` -> negate
            };

            let other_id = if left_is_bool { expr.right } else { expr.left };
            let source = ctx.source_text();
            let other_span = ctx.node(other_id).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let other_start = usize::try_from(other_span.start).unwrap_or(0);
            let other_end = usize::try_from(other_span.end).unwrap_or(0);
            let other_text = source.get(other_start..other_end).unwrap_or("");

            let replacement = if needs_negation {
                format!("!{other_text}")
            } else {
                other_text.to_owned()
            };

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Unnecessary comparison to `{bool_str}` — simplify the expression by removing `{op_str} {bool_str}`"
                ),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some("Simplify the boolean comparison".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Simplify the boolean comparison".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if a node is a boolean literal (`true` or `false`).
fn is_boolean_literal(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.node(node_id), Some(AstNode::BooleanLiteral(_)))
}

/// Extract the boolean value from a boolean literal node.
fn boolean_value(node_id: NodeId, ctx: &LintContext<'_>) -> Option<bool> {
    if let Some(AstNode::BooleanLiteral(lit)) = ctx.node(node_id) {
        Some(lit.value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnnecessaryBooleanLiteralCompare)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_strict_equals_true() {
        let diags = lint("if (x === true) {}");
        assert_eq!(diags.len(), 1, "x === true should be flagged");
    }

    #[test]
    fn test_flags_strict_equals_false() {
        let diags = lint("if (x === false) {}");
        assert_eq!(diags.len(), 1, "x === false should be flagged");
    }

    #[test]
    fn test_flags_loose_equals_true() {
        let diags = lint("if (x == true) {}");
        assert_eq!(diags.len(), 1, "x == true should be flagged");
    }

    #[test]
    fn test_flags_not_equals_false() {
        let diags = lint("if (x !== false) {}");
        assert_eq!(diags.len(), 1, "x !== false should be flagged");
    }

    #[test]
    fn test_allows_comparison_without_boolean_literal() {
        let diags = lint("if (x === y) {}");
        assert!(
            diags.is_empty(),
            "comparison without boolean literal should not be flagged"
        );
    }
}
