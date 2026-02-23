//! Rule: `no-constant-binary-expression`
//!
//! Disallow expressions where the operation is guaranteed to produce a
//! predictable result. For example, `x === null || x === undefined` when
//! using `??` would suffice, or comparisons where one side is always a
//! new object literal (`x === {}` is always `false`).

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags binary expressions that always produce the same result.
#[derive(Debug)]
pub struct NoConstantBinaryExpression;

impl LintRule for NoConstantBinaryExpression {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-constant-binary-expression".to_owned(),
            description: "Disallow expressions where the operation is predictable".to_owned(),
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

        // Check: comparison against a newly constructed object/array/regex
        // e.g. `x === {}`, `x === []`, `x === /re/` — always false for ===,
        // always true for !==
        if expr.operator.is_equality()
            && (is_always_new_value(ctx, expr.left) || is_always_new_value(ctx, expr.right))
        {
            let result_word = if expr.operator == BinaryOperator::StrictInequality
                || expr.operator == BinaryOperator::Inequality
            {
                "true"
            } else {
                "false"
            };
            ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                rule_name: "no-constant-binary-expression".to_owned(),
                message: format!(
                    "This comparison is always `{result_word}` because a new value is created each time"
                ),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression always creates a new value (object/array/regex/class).
fn is_always_new_value(ctx: &LintContext<'_>, id: NodeId) -> bool {
    matches!(
        ctx.node(id),
        Some(
            AstNode::ObjectExpression(_)
                | AstNode::ArrayExpression(_)
                | AstNode::RegExpLiteral(_)
                | AstNode::Class(_)
                | AstNode::Function(_)
                | AstNode::ArrowFunctionExpression(_)
        )
    )
}

#[cfg(test)]
mod tests {
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConstantBinaryExpression)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_comparison_with_object_literal() {
        let diags = lint("if (x === {}) {}");
        assert_eq!(
            diags.len(),
            1,
            "x === empty object should be flagged (always false)"
        );
    }

    #[test]
    fn test_flags_comparison_with_array_literal() {
        let diags = lint("if (x === []) {}");
        assert_eq!(diags.len(), 1, "x === [] should be flagged (always false)");
    }

    #[test]
    fn test_flags_comparison_with_regex() {
        let diags = lint("if (x === /re/) {}");
        assert_eq!(
            diags.len(),
            1,
            "x === /re/ should be flagged (always false)"
        );
    }

    #[test]
    fn test_flags_inequality_with_object() {
        let diags = lint("if (x !== {}) {}");
        assert_eq!(
            diags.len(),
            1,
            "x !== empty object should be flagged (always true)"
        );
    }

    #[test]
    fn test_allows_comparison_with_variable() {
        let diags = lint("if (x === y) {}");
        assert!(
            diags.is_empty(),
            "comparison with variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_comparison_with_null() {
        let diags = lint("if (x === null) {}");
        assert!(
            diags.is_empty(),
            "comparison with null should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_equality() {
        let diags = lint("var x = {} + 1;");
        assert!(
            diags.is_empty(),
            "non-equality with object should not be flagged"
        );
    }
}
