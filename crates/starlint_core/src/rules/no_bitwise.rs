//! Rule: `no-bitwise`
//!
//! Disallow bitwise operators. Bitwise operators are rarely used in
//! JavaScript and are often typos for logical operators (e.g. `&` vs `&&`).

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags bitwise operators.
#[derive(Debug)]
pub struct NoBitwise;

impl LintRule for NoBitwise {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-bitwise".to_owned(),
            description: "Disallow bitwise operators".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression, AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::BinaryExpression(expr) => {
                if is_bitwise_binary(expr.operator) {
                    ctx.report(Diagnostic {
                        rule_name: "no-bitwise".to_owned(),
                        message: "Unexpected use of bitwise operator".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::UnaryExpression(expr) => {
                if expr.operator == UnaryOperator::BitwiseNot {
                    ctx.report(Diagnostic {
                        rule_name: "no-bitwise".to_owned(),
                        message: "Unexpected use of bitwise operator `~`".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Check if a binary operator is a bitwise operator.
const fn is_bitwise_binary(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoBitwise)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_bitwise_and() {
        let diags = lint("var x = a & b;");
        assert_eq!(diags.len(), 1, "bitwise AND should be flagged");
    }

    #[test]
    fn test_flags_bitwise_or() {
        let diags = lint("var x = a | b;");
        assert_eq!(diags.len(), 1, "bitwise OR should be flagged");
    }

    #[test]
    fn test_flags_bitwise_not() {
        let diags = lint("var x = ~a;");
        assert_eq!(diags.len(), 1, "bitwise NOT should be flagged");
    }

    #[test]
    fn test_allows_logical_and() {
        let diags = lint("var x = a && b;");
        assert!(diags.is_empty(), "logical AND should not be flagged");
    }

    #[test]
    fn test_allows_logical_or() {
        let diags = lint("var x = a || b;");
        assert!(diags.is_empty(), "logical OR should not be flagged");
    }
}
