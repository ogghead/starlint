//! Rule: `typescript/restrict-plus-operands`
//!
//! Disallow the `+` operator with mixed string and number literal operands.
//! Adding a string literal to a number literal (or vice versa) is almost always
//! a mistake — the number is silently coerced to a string, producing unexpected
//! concatenation instead of arithmetic.
//!
//! Simplified syntax-only version — full checking requires type information.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/restrict-plus-operands";

/// Flags `+` expressions where one operand is a string literal and the other
/// is a numeric literal.
#[derive(Debug)]
pub struct RestrictPlusOperands;

impl LintRule for RestrictPlusOperands {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `+` operator with mixed string and number literal operands"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(bin) = node else {
            return;
        };

        if bin.operator != BinaryOperator::Addition {
            return;
        }

        if is_mixed_string_number(bin.left, bin.right, ctx) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Unexpected mixed string and number operands for `+` — the number will be \
                 coerced to a string"
                        .to_owned(),
                span: Span::new(bin.span.start, bin.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if one operand is a string literal and the other is a
/// numeric literal (in either order).
fn is_mixed_string_number(left_id: NodeId, right_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let left = ctx.node(left_id);
    let right = ctx.node(right_id);
    let left_is_string = matches!(left, Some(AstNode::StringLiteral(_)));
    let left_is_number = matches!(left, Some(AstNode::NumericLiteral(_)));
    let right_is_string = matches!(right, Some(AstNode::StringLiteral(_)));
    let right_is_number = matches!(right, Some(AstNode::NumericLiteral(_)));

    (left_is_string && right_is_number) || (left_is_number && right_is_string)
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(RestrictPlusOperands, "test.ts");

    #[test]
    fn test_flags_string_plus_number() {
        let diags = lint(r#"const x = "hello" + 42;"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal + number literal should be flagged"
        );
    }

    #[test]
    fn test_flags_number_plus_string() {
        let diags = lint(r#"const x = 42 + "hello";"#);
        assert_eq!(
            diags.len(),
            1,
            "number literal + string literal should be flagged"
        );
    }

    #[test]
    fn test_allows_string_plus_string() {
        let diags = lint(r#"const x = "hello" + " world";"#);
        assert!(
            diags.is_empty(),
            "string + string concatenation should not be flagged"
        );
    }

    #[test]
    fn test_allows_number_plus_number() {
        let diags = lint("const x = 1 + 2;");
        assert!(
            diags.is_empty(),
            "number + number arithmetic should not be flagged"
        );
    }

    #[test]
    fn test_allows_variable_plus_number() {
        let diags = lint("const x = y + 42;");
        assert!(
            diags.is_empty(),
            "variable + number should not be flagged without type info"
        );
    }
}
