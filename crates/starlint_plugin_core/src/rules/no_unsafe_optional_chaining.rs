//! Rule: `no-unsafe-optional-chaining`
//!
//! Disallow use of optional chaining in contexts where `undefined` is not
//! allowed. Using `?.` in arithmetic, `new`, destructuring, or template
//! tags can cause runtime errors because the result might be `undefined`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags unsafe uses of optional chaining that could produce undefined
/// in contexts where it causes errors.
#[derive(Debug)]
pub struct NoUnsafeOptionalChaining;

impl LintRule for NoUnsafeOptionalChaining {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unsafe-optional-chaining".to_owned(),
            description:
                "Disallow use of optional chaining in contexts where undefined is not allowed"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::BinaryExpression,
            AstNodeType::NewExpression,
            AstNodeType::SpreadElement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // `new foo?.bar()` — undefined is not a constructor
            AstNode::NewExpression(new_expr) => {
                if contains_optional_chain(new_expr.callee, ctx) {
                    ctx.report(Diagnostic {
                        rule_name: "no-unsafe-optional-chaining".to_owned(),
                        message: "Unsafe use of optional chaining in `new` expression".to_owned(),
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            // Arithmetic operations on optional chain: `foo?.bar + 1`
            AstNode::BinaryExpression(bin) => {
                if matches!(
                    bin.operator,
                    starlint_ast::operator::BinaryOperator::Addition
                        | starlint_ast::operator::BinaryOperator::Subtraction
                        | starlint_ast::operator::BinaryOperator::Multiplication
                        | starlint_ast::operator::BinaryOperator::Division
                        | starlint_ast::operator::BinaryOperator::Remainder
                        | starlint_ast::operator::BinaryOperator::Exponential
                        | starlint_ast::operator::BinaryOperator::ShiftLeft
                        | starlint_ast::operator::BinaryOperator::ShiftRight
                        | starlint_ast::operator::BinaryOperator::ShiftRightZeroFill
                        | starlint_ast::operator::BinaryOperator::BitwiseOR
                        | starlint_ast::operator::BinaryOperator::BitwiseXOR
                        | starlint_ast::operator::BinaryOperator::BitwiseAnd
                ) {
                    if contains_optional_chain(bin.left, ctx) {
                        report_arithmetic(bin.span, ctx);
                    }
                    if contains_optional_chain(bin.right, ctx) {
                        report_arithmetic(bin.span, ctx);
                    }
                }
            }
            // Spread: `[...foo?.bar]` — undefined is not iterable
            AstNode::SpreadElement(spread) => {
                if contains_optional_chain(spread.argument, ctx) {
                    ctx.report(Diagnostic {
                        rule_name: "no-unsafe-optional-chaining".to_owned(),
                        message: "Unsafe use of optional chaining in spread element".to_owned(),
                        span: Span::new(spread.span.start, spread.span.end),
                        severity: Severity::Error,
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

/// Check if an expression directly contains optional chaining.
fn contains_optional_chain(id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(node) = ctx.node(id) else {
        return false;
    };
    matches!(node, AstNode::ChainExpression(_))
}

/// Report unsafe optional chaining in arithmetic context.
fn report_arithmetic(span: starlint_ast::types::Span, ctx: &mut LintContext<'_>) {
    ctx.report(Diagnostic {
        rule_name: "no-unsafe-optional-chaining".to_owned(),
        message: "Unsafe use of optional chaining in arithmetic operation".to_owned(),
        span: Span::new(span.start, span.end),
        severity: Severity::Error,
        help: None,
        fix: None,
        labels: vec![],
    });
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeOptionalChaining)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_with_optional_chain() {
        let diags = lint("new (foo?.bar)();");
        assert_eq!(diags.len(), 1, "new with optional chain should be flagged");
    }

    #[test]
    fn test_flags_arithmetic_with_optional_chain() {
        let diags = lint("var x = foo?.bar + 1;");
        assert_eq!(
            diags.len(),
            1,
            "arithmetic with optional chain should be flagged"
        );
    }

    #[test]
    fn test_allows_safe_optional_chain() {
        let diags = lint("var x = foo?.bar;");
        assert!(
            diags.is_empty(),
            "simple optional chain should not be flagged"
        );
    }

    #[test]
    fn test_allows_optional_chain_in_condition() {
        let diags = lint("if (foo?.bar) {}");
        assert!(
            diags.is_empty(),
            "optional chain in condition should not be flagged"
        );
    }

    #[test]
    fn test_allows_nullish_coalescing() {
        let diags = lint("var x = (foo?.bar ?? 0) + 1;");
        assert!(
            diags.is_empty(),
            "optional chain with nullish coalescing should not be flagged"
        );
    }
}
