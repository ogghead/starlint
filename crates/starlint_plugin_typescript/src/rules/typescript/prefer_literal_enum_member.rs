//! Rule: `typescript/prefer-literal-enum-member`
//!
//! Prefer literal values in enum members rather than computed values. Enum
//! members with computed initializers (identifiers, call expressions, binary
//! expressions, etc.) make the enum harder to reason about at a glance and can
//! introduce unexpected runtime behavior. Literal values (strings, numbers,
//! unary negation of a number) are always safe and clear.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags enum members whose initializers are not literal values.
#[derive(Debug)]
pub struct PreferLiteralEnumMember;

impl LintRule for PreferLiteralEnumMember {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-literal-enum-member".to_owned(),
            description: "Prefer literal values in enum members".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSEnumMember])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSEnumMember(member) = node else {
            return;
        };

        let Some(init_id) = member.initializer else {
            // No initializer — auto-incremented; this is fine.
            return;
        };

        if is_literal_value(init_id, ctx) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/prefer-literal-enum-member".to_owned(),
            message: "Enum member should be initialized with a literal value".to_owned(),
            span: Span::new(member.span.start, member.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check whether an expression is a valid literal enum initializer.
///
/// Accepted forms:
/// - String literal (`"hello"`)
/// - Numeric literal (`42`)
/// - Boolean literal (`true`, `false`)
/// - Template literal with no expressions (`` `hello` ``)
/// - Unary negation of a numeric literal (`-1`)
fn is_literal_value(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(expr) = ctx.node(expr_id) else {
        return false;
    };
    match expr {
        AstNode::StringLiteral(_) | AstNode::NumericLiteral(_) | AstNode::BooleanLiteral(_) => true,
        AstNode::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        AstNode::UnaryExpression(unary) => {
            unary.operator == UnaryOperator::UnaryNegation
                && matches!(ctx.node(unary.argument), Some(AstNode::NumericLiteral(_)))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferLiteralEnumMember)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_function_call_initializer() {
        let diags = lint("enum E { A = foo() }");
        assert_eq!(
            diags.len(),
            1,
            "enum member with function call initializer should be flagged"
        );
    }

    #[test]
    fn test_flags_identifier_initializer() {
        let diags = lint("enum E { A = x }");
        assert_eq!(
            diags.len(),
            1,
            "enum member with identifier initializer should be flagged"
        );
    }

    #[test]
    fn test_allows_numeric_literal() {
        let diags = lint("enum E { A = 1 }");
        assert!(
            diags.is_empty(),
            "enum member with numeric literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_literal() {
        let diags = lint("enum E { A = \"hello\" }");
        assert!(
            diags.is_empty(),
            "enum member with string literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_initializer() {
        let diags = lint("enum E { A }");
        assert!(
            diags.is_empty(),
            "enum member without initializer should not be flagged"
        );
    }

    #[test]
    fn test_allows_negative_number() {
        let diags = lint("enum E { A = -1 }");
        assert!(
            diags.is_empty(),
            "enum member with negative number literal should not be flagged"
        );
    }
}
