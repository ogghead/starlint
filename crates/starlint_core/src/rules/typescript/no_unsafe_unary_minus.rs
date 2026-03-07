//! Rule: `typescript/no-unsafe-unary-minus`
//!
//! Disallow unary minus on non-numeric types. Applying the unary minus
//! operator to a non-numeric value produces `NaN`, which is almost always
//! a bug. This rule flags obvious cases like `-"string"`, `-true`, `-false`,
//! `-null`, `-undefined`, `-{}`, and `-[]`.
//!
//! Since we cannot perform full type checking, only literal expressions
//! and well-known non-numeric identifiers are detected.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

/// Flags unary minus applied to obviously non-numeric values.
#[derive(Debug)]
pub struct NoUnsafeUnaryMinus;

impl LintRule for NoUnsafeUnaryMinus {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-unary-minus".to_owned(),
            description: "Disallow unary minus on non-numeric types".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::UnaryExpression(expr) = node else {
            return;
        };

        if expr.operator != UnaryOperator::UnaryNegation {
            return;
        }

        if let Some(description) = ctx.node(expr.argument).and_then(is_non_numeric_operand) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-unary-minus".to_owned(),
                message: format!("Unary minus on {description} produces `NaN`"),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check whether an expression is obviously non-numeric.
///
/// Returns a human-readable description of the non-numeric type when the
/// operand is clearly not a number, or `None` when it could be numeric.
fn is_non_numeric_operand(node: &AstNode) -> Option<&'static str> {
    match node {
        AstNode::StringLiteral(_) => Some("a string literal"),
        AstNode::BooleanLiteral(_) => Some("a boolean literal"),
        AstNode::NullLiteral(_) => Some("`null`"),
        AstNode::ObjectExpression(_) => Some("an object literal"),
        AstNode::ArrayExpression(_) => Some("an array literal"),
        AstNode::IdentifierReference(ident) if ident.name == "undefined" => Some("`undefined`"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeUnaryMinus)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_negate_string() {
        let diags = lint(r#"let x = -"hello";"#);
        assert_eq!(
            diags.len(),
            1,
            "negating a string literal should be flagged"
        );
    }

    #[test]
    fn test_flags_negate_boolean() {
        let diags = lint("let x = -true;");
        assert_eq!(
            diags.len(),
            1,
            "negating a boolean literal should be flagged"
        );
    }

    #[test]
    fn test_flags_negate_null() {
        let diags = lint("let x = -null;");
        assert_eq!(diags.len(), 1, "negating null should be flagged");
    }

    #[test]
    fn test_flags_negate_object() {
        let diags = lint("let x = -{};");
        assert_eq!(
            diags.len(),
            1,
            "negating an object literal should be flagged"
        );
    }

    #[test]
    fn test_allows_negate_number() {
        let diags = lint("let x = -42;");
        assert!(
            diags.is_empty(),
            "negating a number literal should not be flagged"
        );
    }
}
