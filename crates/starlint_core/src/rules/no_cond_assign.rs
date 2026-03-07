//! Rule: `no-cond-assign`
//!
//! Disallow assignment operators in conditional expressions. Using `=` instead
//! of `==` or `===` in conditions is a common mistake: `if (x = 5)` assigns 5
//! to x instead of comparing.

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags assignment expressions used directly in conditions.
#[derive(Debug)]
pub struct NoCondAssign;

impl LintRule for NoCondAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-cond-assign".to_owned(),
            description: "Disallow assignment operators in conditional expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ConditionalExpression,
            AstNodeType::DoWhileStatement,
            AstNodeType::ForStatement,
            AstNodeType::IfStatement,
            AstNodeType::WhileStatement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::IfStatement(if_stmt) => {
                check_condition(if_stmt.test, false, ctx);
            }
            AstNode::WhileStatement(while_stmt) => {
                check_condition(while_stmt.test, false, ctx);
            }
            AstNode::DoWhileStatement(do_while) => {
                check_condition(do_while.test, false, ctx);
            }
            AstNode::ForStatement(for_stmt) => {
                if let Some(test) = for_stmt.test {
                    check_condition(test, false, ctx);
                }
            }
            AstNode::ConditionalExpression(cond) => {
                check_condition(cond.test, true, ctx);
            }
            _ => {}
        }
    }
}

/// Check if an expression (used as a condition) is an assignment.
///
/// When `is_ternary` is true, parenthesized assignments are considered
/// intentional (consistent with `ESLint`'s default "except-parens" mode).
/// For if/while/for/do-while the condition is always syntactically wrapped
/// in `()`, so extra parens would mean double-parens `((x = 5))`.
fn check_condition(test_id: NodeId, is_ternary: bool, ctx: &mut LintContext<'_>) {
    let Some(AstNode::AssignmentExpression(assign)) = ctx.node(test_id) else {
        return;
    };

    // Extract spans before mutably borrowing ctx
    let assign_span = assign.span;

    // For ternary expressions, the converter unwraps ParenthesizedExpression
    // but we can detect it was parenthesized by checking if `(` precedes
    // the assignment span. Statement conditions (if/while/etc.) always have
    // syntactic parens so this check only applies to ternaries.
    if is_ternary {
        #[allow(clippy::as_conversions)]
        let start = assign_span.start as usize;
        if let Some(prev_byte) = start
            .checked_sub(1)
            .and_then(|i| ctx.source_text().as_bytes().get(i))
        {
            if *prev_byte == b'(' {
                return;
            }
        }
    }

    let left_id = assign.left;
    let right_id = assign.right;

    // Fix: replace `=` with `===`
    #[allow(clippy::as_conversions)]
    let fix = {
        let source = ctx.source_text();
        let left_span = ctx.node(left_id).map_or(
            starlint_ast::types::Span::new(0, 0),
            starlint_ast::AstNode::span,
        );
        let right_span = ctx.node(right_id).map_or(
            starlint_ast::types::Span::new(0, 0),
            starlint_ast::AstNode::span,
        );
        let left_text = source
            .get(left_span.start as usize..left_span.end as usize)
            .unwrap_or("");
        let right_text = source
            .get(right_span.start as usize..right_span.end as usize)
            .unwrap_or("");
        let replacement = format!("{left_text} === {right_text}");
        Some(Fix {
            kind: FixKind::SafeFix,
            message: format!("Replace with `{replacement}`"),
            edits: vec![Edit {
                span: Span::new(assign_span.start, assign_span.end),
                replacement,
            }],
            is_snippet: false,
        })
    };

    ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
        rule_name: "no-cond-assign".to_owned(),
        message: "Unexpected assignment in conditional expression".to_owned(),
        span: Span::new(assign_span.start, assign_span.end),
        severity: Severity::Error,
        help: Some("Did you mean `===` instead of `=`?".to_owned()),
        fix,
        labels: vec![],
    });
}

#[cfg(test)]
mod tests {
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoCondAssign)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_assignment_in_if() {
        let diags = lint("if (x = 5) {}");
        assert_eq!(diags.len(), 1, "assignment in if should be flagged");
    }

    #[test]
    fn test_flags_assignment_in_while() {
        let diags = lint("while (x = 5) {}");
        assert_eq!(diags.len(), 1, "assignment in while should be flagged");
    }

    #[test]
    fn test_flags_assignment_in_do_while() {
        let diags = lint("do {} while (x = 5);");
        assert_eq!(diags.len(), 1, "assignment in do-while should be flagged");
    }

    #[test]
    fn test_flags_assignment_in_for() {
        let diags = lint("for (;x = 5;) {}");
        assert_eq!(
            diags.len(),
            1,
            "assignment in for condition should be flagged"
        );
    }

    #[test]
    fn test_allows_parenthesized_assignment_in_ternary() {
        // Parenthesized assignments are intentional — consistent with ESLint's
        // default "except-parens" mode
        let diags = lint("var y = (x = 5) ? 1 : 2;");
        assert!(
            diags.is_empty(),
            "parenthesized assignment in ternary should not be flagged"
        );
    }

    #[test]
    fn test_allows_comparison() {
        let diags = lint("if (x === 5) {}");
        assert!(diags.is_empty(), "comparison should not be flagged");
    }

    #[test]
    fn test_allows_assignment_outside_condition() {
        let diags = lint("var x = 5;");
        assert!(
            diags.is_empty(),
            "assignment outside condition should not be flagged"
        );
    }
}
