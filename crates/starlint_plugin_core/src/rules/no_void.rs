//! Rule: `no-void`
//!
//! Disallow the `void` operator. The `void` operator is rarely needed
//! and can be confusing.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags use of the `void` operator.
#[derive(Debug)]
pub struct NoVoid;

impl LintRule for NoVoid {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-void".to_owned(),
            description: "Disallow the `void` operator".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::UnaryExpression(unary) = node else {
            return;
        };

        if unary.operator == UnaryOperator::Void {
            ctx.report(Diagnostic {
                rule_name: "no-void".to_owned(),
                message: "Expected `undefined` instead of `void`".to_owned(),
                span: Span::new(unary.span.start, unary.span.end),
                severity: Severity::Warning,
                help: Some("Replace `void` expression with `undefined`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace with `undefined`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(unary.span.start, unary.span.end),
                        replacement: "undefined".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoVoid);

    #[test]
    fn test_flags_void_operator() {
        let diags = lint("var x = void 0;");
        assert_eq!(diags.len(), 1, "void operator should be flagged");
    }

    #[test]
    fn test_allows_undefined() {
        let diags = lint("var x = undefined;");
        assert!(diags.is_empty(), "undefined should not be flagged");
    }
}
