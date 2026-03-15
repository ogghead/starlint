//! Rule: `prefer-switch` (unicorn)
//!
//! Flags chains of `if`/`else if` that compare the same variable with `===`.
//! When 3+ conditions compare the same identifier with strict equality, a
//! `switch` statement is clearer.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Minimum number of `===` branches on the same identifier before flagging.
const MIN_CASES: u32 = 3;

/// Flags if-else-if chains that could be replaced with a `switch` statement.
#[derive(Debug)]
pub struct PreferSwitch;

impl LintRule for PreferSwitch {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-switch".to_owned(),
            description: "Prefer `switch` over multiple `===` comparisons on the same variable"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        // Only trigger on the top-level `if` of a chain. If this `if` is itself
        // the `alternate` of a parent `if`, skip it to avoid duplicate reports.
        // We detect this by checking if the source text immediately before the
        // `if` keyword ends with `else`.
        if is_else_if_branch(if_stmt.span.start, ctx.source_text()) {
            return;
        }

        // Extract the identifier being compared in the first branch.
        let Some(first_ident) = strict_eq_identifier(if_stmt.test, ctx) else {
            return;
        };

        // Walk the else-if chain, counting branches that compare the same identifier.
        let mut count: u32 = 1;
        let mut current_alt = if_stmt.alternate;
        while let Some(alt_id) = current_alt {
            if let Some(AstNode::IfStatement(else_if)) = ctx.node(alt_id) {
                if let Some(ident) = strict_eq_identifier(else_if.test, ctx) {
                    if ident == first_ident {
                        count = count.saturating_add(1);
                        current_alt = else_if.alternate;
                        continue;
                    }
                }
            }
            break;
        }

        if count >= MIN_CASES {
            ctx.report(Diagnostic {
                rule_name: "prefer-switch".to_owned(),
                message: format!(
                    "Use a `switch` statement instead of {count} `if`/`else if` comparisons on `{first_ident}`"
                ),
                span: Span::new(if_stmt.span.start, if_stmt.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if this `if` statement is the `alternate` of a parent `if`.
/// Looks backwards from the `if` keyword for the word `else`.
fn is_else_if_branch(if_start: u32, source: &str) -> bool {
    let start = usize::try_from(if_start).unwrap_or(0);
    // Walk backwards over whitespace to find `else`.
    let before = source.get(..start).unwrap_or("");
    let trimmed = before.trim_end();
    trimmed.ends_with("else")
}

/// If the expression is a `BinaryExpression` with `===` and one side is an
/// `Identifier`, return that identifier's name.
fn strict_eq_identifier(expr_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let Some(AstNode::BinaryExpression(bin)) = ctx.node(expr_id) else {
        return None;
    };

    if bin.operator != BinaryOperator::StrictEquality {
        return None;
    }

    // Check left side first, then right.
    if let Some(AstNode::IdentifierReference(id)) = ctx.node(bin.left) {
        return Some(id.name.clone());
    }
    if let Some(AstNode::IdentifierReference(id)) = ctx.node(bin.right) {
        return Some(id.name.clone());
    }

    None
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferSwitch);

    #[test]
    fn test_flags_three_strict_equality_branches() {
        let diags = lint("if (x === 1) {} else if (x === 2) {} else if (x === 3) {}");
        assert!(
            !diags.is_empty(),
            "3+ strict equality branches on same variable should be flagged"
        );
    }

    #[test]
    fn test_flags_four_branches() {
        let diags = lint(
            "if (x === 'a') {} else if (x === 'b') {} else if (x === 'c') {} else if (x === 'd') {}",
        );
        assert!(
            !diags.is_empty(),
            "4 strict equality branches should be flagged"
        );
    }

    #[test]
    fn test_allows_only_two_branches() {
        let diags = lint("if (x === 1) {} else if (x === 2) {}");
        assert!(diags.is_empty(), "only 2 branches should not be flagged");
    }

    #[test]
    fn test_allows_different_variables() {
        let diags = lint("if (x === 1) {} else if (y === 2) {} else if (z === 3) {}");
        assert!(
            diags.is_empty(),
            "different variables should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_strict_equality() {
        let diags = lint("if (x == 1) {} else if (x == 2) {} else if (x == 3) {}");
        assert!(diags.is_empty(), "loose equality should not be flagged");
    }

    #[test]
    fn test_allows_mixed_operators() {
        let diags = lint("if (x === 1) {} else if (x > 2) {} else if (x === 3) {}");
        assert!(diags.is_empty(), "mixed operators should break the chain");
    }

    #[test]
    fn test_allows_simple_if() {
        let diags = lint("if (x === 1) {}");
        assert!(diags.is_empty(), "single if should not be flagged");
    }

    #[test]
    fn test_allows_if_else_no_chain() {
        let diags = lint("if (x === 1) {} else {}");
        assert!(
            diags.is_empty(),
            "if-else without chain should not be flagged"
        );
    }

    #[test]
    fn test_identifier_on_right_side() {
        let diags = lint("if (1 === x) {} else if (2 === x) {} else if (3 === x) {}");
        assert!(
            !diags.is_empty(),
            "identifier on right side of === should also be detected"
        );
    }
}
