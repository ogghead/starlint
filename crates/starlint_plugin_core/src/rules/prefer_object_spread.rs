//! Rule: `prefer-object-spread`
//!
//! Disallow using `Object.assign()` with an object literal as the first argument.
//! Prefer `{ ...foo }` over `Object.assign({}, foo)`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `Object.assign({}, ...)` that can use spread.
#[derive(Debug)]
pub struct PreferObjectSpread;

impl LintRule for PreferObjectSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-object-spread".to_owned(),
            description: "Disallow using `Object.assign` with object literal first argument"
                .to_owned(),
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

        // Must be Object.assign(...)
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "assign" {
            return;
        }

        if !matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Object")
        {
            return;
        }

        // Must have at least one argument, and the first must be an empty object literal
        if let Some(&first_arg_id) = call.arguments.first() {
            let is_empty_object = match ctx.node(first_arg_id) {
                Some(AstNode::ObjectExpression(obj)) => obj.properties.is_empty(),
                _ => false,
            };

            if is_empty_object {
                let source = ctx.source_text();
                let mut parts = Vec::new();
                for &arg_id in call.arguments.iter().skip(1) {
                    if let Some(arg_node) = ctx.node(arg_id) {
                        let span = arg_node.span();
                        let s = usize::try_from(span.start).unwrap_or(0);
                        let e = usize::try_from(span.end).unwrap_or(0);
                        let text = source.get(s..e).unwrap_or("");
                        parts.push(format!("...{text}"));
                    }
                }
                let replacement = if parts.is_empty() {
                    "{}".to_owned()
                } else {
                    format!("{{ {} }}", parts.join(", "))
                };

                ctx.report(Diagnostic {
                    rule_name: "prefer-object-spread".to_owned(),
                    message: "Use an object spread instead of `Object.assign` with empty object"
                        .to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace with object spread".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Replace with object spread".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferObjectSpread);

    #[test]
    fn test_flags_object_assign_empty_first() {
        let diags = lint("var x = Object.assign({}, foo);");
        assert_eq!(
            diags.len(),
            1,
            "Object.assign with empty object first should be flagged"
        );
    }

    #[test]
    fn test_allows_object_assign_non_empty_first() {
        let diags = lint("var x = Object.assign({ a: 1 }, foo);");
        assert!(
            diags.is_empty(),
            "Object.assign with non-empty first should not be flagged"
        );
    }

    #[test]
    fn test_allows_spread_syntax() {
        let diags = lint("var x = { ...foo };");
        assert!(diags.is_empty(), "spread syntax should not be flagged");
    }
}
