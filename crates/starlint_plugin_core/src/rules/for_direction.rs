//! Rule: `for-direction`
//!
//! Enforce `for` loop update clause moving the counter in the right direction.
//! A `for` loop with a stop condition that can never be reached (e.g.
//! `for (i = 0; i < 10; i--)`) is almost certainly a bug.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{AssignmentOperator, BinaryOperator, UpdateOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `for` loops whose update clause moves the counter away from the stop
/// condition, making the loop either infinite or immediately terminating.
#[derive(Debug)]
pub struct ForDirection;

impl LintRule for ForDirection {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "for-direction".to_owned(),
            description:
                "Enforce `for` loop update clause moving the counter toward the stop condition"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ForStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ForStatement(stmt) = node else {
            return;
        };

        // We need both a test (stop condition) and an update clause.
        let (Some(test_id), Some(update_id)) = (stmt.test, stmt.update) else {
            return;
        };

        let Some(test_node) = ctx.node(test_id) else {
            return;
        };
        let Some(update_node) = ctx.node(update_id) else {
            return;
        };

        // Extract the comparison operator and the counter name from the test.
        let Some((counter_name, direction)) = extract_test_direction(ctx, test_node) else {
            return;
        };

        // Check whether the update moves the counter in the wrong direction.
        if moves_wrong_direction(ctx, update_node, &counter_name, direction) {
            // Fix: swap the update direction
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let update_span = update_node.span();
                source
                    .get(update_span.start as usize..update_span.end as usize)
                    .and_then(|update_text| {
                        let replacement = swap_update_direction(update_text)?;
                        Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Replace `{update_text}` with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(update_span.start, update_span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        })
                    })
            };

            ctx.report(Diagnostic {
                rule_name: "for-direction".to_owned(),
                message: format!(
                    "The update clause in this `for` loop moves the counter `{counter_name}` in the wrong direction"
                ),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Error,
                help: Some(
                    "The loop counter should move toward the stop condition, not away from it"
                        .to_owned(),
                ),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Which direction the counter must move based on the comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CounterDirection {
    /// Counter must increase (e.g. `i < 10`).
    Increasing,
    /// Counter must decrease (e.g. `i > 0`).
    Decreasing,
}

/// Given a test expression like `i < 10`, extract the counter variable name
/// and the required direction. Returns `None` if the test is not a simple
/// comparison with an identifier on one side.
fn extract_test_direction(
    ctx: &LintContext<'_>,
    test: &AstNode,
) -> Option<(String, CounterDirection)> {
    let AstNode::BinaryExpression(bin) = test else {
        return None;
    };

    let left_node = ctx.node(bin.left)?;

    match bin.operator {
        // `counter < x` or `counter <= x` → counter must increase
        BinaryOperator::LessThan | BinaryOperator::LessEqualThan => {
            identifier_name(left_node).map(|name| (name.to_owned(), CounterDirection::Increasing))
        }
        // `counter > x` or `counter >= x` → counter must decrease
        BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan => {
            identifier_name(left_node).map(|name| (name.to_owned(), CounterDirection::Decreasing))
        }
        _ => None,
    }
}

/// Extract a plain identifier name from an `AstNode`.
fn identifier_name(node: &AstNode) -> Option<&str> {
    if let AstNode::IdentifierReference(ident) = node {
        Some(ident.name.as_str())
    } else {
        None
    }
}

/// Check if the update expression moves the named counter in the wrong direction.
fn moves_wrong_direction(
    ctx: &LintContext<'_>,
    update: &AstNode,
    counter_name: &str,
    required: CounterDirection,
) -> bool {
    match update {
        // `i++` or `i--`
        AstNode::UpdateExpression(upd) => {
            let target_name = ctx.node(upd.argument).and_then(identifier_name);
            if target_name != Some(counter_name) {
                return false;
            }
            match upd.operator {
                UpdateOperator::Increment => required == CounterDirection::Decreasing,
                UpdateOperator::Decrement => required == CounterDirection::Increasing,
            }
        }
        // `i += 1` or `i -= 1`
        AstNode::AssignmentExpression(assign) => {
            let target_name = ctx.node(assign.left).and_then(identifier_name);
            if target_name != Some(counter_name) {
                return false;
            }
            let right_positive = ctx.node(assign.right).is_some_and(is_positive_numeric);
            match assign.operator {
                AssignmentOperator::Addition => {
                    right_positive && required == CounterDirection::Decreasing
                }
                AssignmentOperator::Subtraction => {
                    right_positive && required == CounterDirection::Increasing
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Check if an `AstNode` is a positive numeric literal (not zero).
fn is_positive_numeric(node: &AstNode) -> bool {
    if let AstNode::NumericLiteral(lit) = node {
        lit.value > 0.0
    } else {
        false
    }
}

/// Swap the direction of an update expression in source text.
/// `i++` → `i--`, `i--` → `i++`, `i += 1` → `i -= 1`, `i -= 1` → `i += 1`
fn swap_update_direction(text: &str) -> Option<String> {
    if text.ends_with("++") {
        Some(format!("{}--", &text[..text.len().saturating_sub(2)]))
    } else if text.ends_with("--") {
        Some(format!("{}++", &text[..text.len().saturating_sub(2)]))
    } else if let Some(rest) = text.strip_prefix("++") {
        Some(format!("--{rest}"))
    } else if let Some(rest) = text.strip_prefix("--") {
        Some(format!("++{rest}"))
    } else if let Some(pos) = text.find("+=") {
        let mut result = String::with_capacity(text.len());
        result.push_str(&text[..pos]);
        result.push_str("-=");
        result.push_str(&text[pos.saturating_add(2)..]);
        Some(result)
    } else if let Some(pos) = text.find("-=") {
        let mut result = String::with_capacity(text.len());
        result.push_str(&text[..pos]);
        result.push_str("+=");
        result.push_str(&text[pos.saturating_add(2)..]);
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ForDirection)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_correct_increment() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert!(diags.is_empty(), "correct for loop should not be flagged");
    }

    #[test]
    fn test_correct_decrement() {
        let diags = lint("for (let i = 10; i > 0; i--) {}");
        assert!(
            diags.is_empty(),
            "correct decrement loop should not be flagged"
        );
    }

    #[test]
    fn test_wrong_direction_increment() {
        let diags = lint("for (let i = 0; i < 10; i--) {}");
        assert_eq!(diags.len(), 1, "decrementing toward < should be flagged");
    }

    #[test]
    fn test_wrong_direction_decrement() {
        let diags = lint("for (let i = 10; i > 0; i++) {}");
        assert_eq!(diags.len(), 1, "incrementing toward > should be flagged");
    }

    #[test]
    fn test_wrong_direction_assignment_add() {
        let diags = lint("for (let i = 10; i > 0; i += 1) {}");
        assert_eq!(diags.len(), 1, "adding toward > should be flagged");
    }

    #[test]
    fn test_wrong_direction_assignment_sub() {
        let diags = lint("for (let i = 0; i < 10; i -= 1) {}");
        assert_eq!(diags.len(), 1, "subtracting toward < should be flagged");
    }

    #[test]
    fn test_correct_assignment_add() {
        let diags = lint("for (let i = 0; i < 10; i += 1) {}");
        assert!(diags.is_empty(), "adding toward < should not be flagged");
    }

    #[test]
    fn test_no_test_clause() {
        let diags = lint("for (let i = 0; ; i++) {}");
        assert!(diags.is_empty(), "no test clause should not be flagged");
    }

    #[test]
    fn test_no_update_clause() {
        let diags = lint("for (let i = 0; i < 10; ) {}");
        assert!(diags.is_empty(), "no update clause should not be flagged");
    }

    #[test]
    fn test_lte_wrong_direction() {
        let diags = lint("for (let i = 0; i <= 10; i--) {}");
        assert_eq!(diags.len(), 1, "<= with decrement should be flagged");
    }

    #[test]
    fn test_gte_wrong_direction() {
        let diags = lint("for (let i = 10; i >= 0; i++) {}");
        assert_eq!(diags.len(), 1, ">= with increment should be flagged");
    }
}
