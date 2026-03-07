//! Rule: `prefer-exponentiation-operator`
//!
//! Disallow the use of `Math.pow()` in favor of the `**` operator.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

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
                    let f_start = usize::try_from(first_node.span().start).unwrap_or(0);
                    let f_end = usize::try_from(first_node.span().end).unwrap_or(0);
                    let s_start = usize::try_from(second_node.span().start).unwrap_or(0);
                    let s_end = usize::try_from(second_node.span().end).unwrap_or(0);
                    let first_text = source.get(f_start..f_end).unwrap_or("");
                    let second_text = source.get(s_start..s_end).unwrap_or("");
                    Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Use `**` operator".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement: format!("{first_text} ** {second_text}"),
                        }],
                        is_snippet: false,
                    })
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferExponentiationOperator)];
        lint_source(source, "test.js", &rules)
    }

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
