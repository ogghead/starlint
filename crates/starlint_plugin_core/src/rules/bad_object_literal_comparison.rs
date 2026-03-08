//! Rule: `bad-object-literal-comparison` (OXC)
//!
//! Catch comparisons like `x === {}` or `x === []` which are always false
//! because object/array literals create new references.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags equality comparisons against object or array literals.
#[derive(Debug)]
pub struct BadObjectLiteralComparison;

impl LintRule for BadObjectLiteralComparison {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-object-literal-comparison".to_owned(),
            description: "Catch `x === {}` or `x === []` (always false)".to_owned(),
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

        if !expr.operator.is_equality() {
            return;
        }

        let left_node = ctx.node(expr.left);
        let right_node = ctx.node(expr.right);

        let left_is_literal = left_node.is_some_and(is_object_or_array_literal);
        let right_is_literal = right_node.is_some_and(is_object_or_array_literal);

        // Flag if either side is an object/array literal (and the other is not)
        if left_is_literal || right_is_literal {
            let kind_name = if left_is_literal {
                left_node.map_or("a literal", literal_kind_name)
            } else {
                right_node.map_or("a literal", literal_kind_name)
            };
            ctx.report(Diagnostic {
                rule_name: "bad-object-literal-comparison".to_owned(),
                message: format!(
                    "Comparison against {kind_name} literal is always false — \
                     object/array literals create new references"
                ),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is an object or array literal.
const fn is_object_or_array_literal(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::ObjectExpression(_) | AstNode::ArrayExpression(_)
    )
}

/// Get a human-readable name for the literal kind.
const fn literal_kind_name(node: &AstNode) -> &'static str {
    match node {
        AstNode::ObjectExpression(_) => "an object",
        AstNode::ArrayExpression(_) => "an array",
        _ => "a literal",
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(BadObjectLiteralComparison)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_object_strict_equality() {
        let diags = lint("if (x === {}) {}");
        assert_eq!(diags.len(), 1, "x === empty object should be flagged");
    }

    #[test]
    fn test_flags_array_strict_equality() {
        let diags = lint("if (x === []) {}");
        assert_eq!(diags.len(), 1, "x === empty array should be flagged");
    }

    #[test]
    fn test_flags_loose_equality() {
        let diags = lint("if (x == {}) {}");
        assert_eq!(diags.len(), 1, "x == empty object should be flagged");
    }

    #[test]
    fn test_flags_inequality() {
        let diags = lint("if (x !== []) {}");
        assert_eq!(diags.len(), 1, "x !== empty array should be flagged");
    }

    #[test]
    fn test_allows_string_comparison() {
        let diags = lint("if (x === 'hello') {}");
        assert!(diags.is_empty(), "string comparison should not be flagged");
    }

    #[test]
    fn test_allows_variable_comparison() {
        let diags = lint("if (x === y) {}");
        assert!(
            diags.is_empty(),
            "variable comparison should not be flagged"
        );
    }
}
