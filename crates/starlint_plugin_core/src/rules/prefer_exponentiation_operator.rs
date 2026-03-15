//! Rule: `prefer-exponentiation-operator`
//!
//! Disallow the use of `Math.pow()` in favor of the `**` operator.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::fix_utils::source_text_for_span;
use starlint_rule_framework::{FixBuilder, LintContext, LintRule};

/// Flags `Math.pow()` calls.
#[derive(Debug)]
pub struct PreferExponentiationOperator;

impl LintRule for PreferExponentiationOperator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-exponentiation-operator".to_owned(),
            description: "Disallow the use of `Math.pow` in favor of `**`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "pow" {
            return;
        }

        if matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Math")
        {
            let fix = call.arguments.first().zip(call.arguments.get(1)).and_then(
                |(first_id, second_id)| {
                    let first_node = ctx.node(*first_id)?;
                    let second_node = ctx.node(*second_id)?;
                    let source = ctx.source_text();
                    let first_span = first_node.span();
                    let second_span = second_node.span();
                    let first_text =
                        source_text_for_span(source, Span::new(first_span.start, first_span.end))
                            .unwrap_or("");
                    let second_text =
                        source_text_for_span(source, Span::new(second_span.start, second_span.end))
                            .unwrap_or("");
                    FixBuilder::new("Use `**` operator", FixKind::SafeFix)
                        .replace(
                            Span::new(call.span.start, call.span.end),
                            format!("{first_text} ** {second_text}"),
                        )
                        .build()
                },
            );

            ctx.report(Diagnostic {
                rule_name: "prefer-exponentiation-operator".to_owned(),
                message: "Use the `**` operator instead of `Math.pow()`".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `**` operator".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferExponentiationOperator);

    #[test]
    fn test_flags_math_pow() {
        let diags = lint("var x = Math.pow(2, 3);");
        assert_eq!(diags.len(), 1, "Math.pow() should be flagged");
    }

    #[test]
    fn test_allows_exponentiation_operator() {
        let diags = lint("var x = 2 ** 3;");
        assert!(diags.is_empty(), "** operator should not be flagged");
    }

    #[test]
    fn test_allows_other_math_methods() {
        let diags = lint("var x = Math.floor(3.14);");
        assert!(diags.is_empty(), "other Math methods should not be flagged");
    }
}
