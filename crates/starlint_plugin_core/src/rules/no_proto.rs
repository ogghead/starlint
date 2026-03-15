//! Rule: `no-proto`
//!
//! Disallow the use of the `__proto__` property. Use `Object.getPrototypeOf`
//! and `Object.setPrototypeOf` instead.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags usage of the deprecated `__proto__` property.
#[derive(Debug)]
pub struct NoProto;

impl LintRule for NoProto {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-proto".to_owned(),
            description: "Disallow the use of the `__proto__` property".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticMemberExpression(member) = node else {
            return;
        };

        if member.property.as_str() == "__proto__" {
            #[allow(clippy::as_conversions)]
            let fix = ctx
                .node(member.object)
                .and_then(|obj_node| {
                    let obj_span = obj_node.span();
                    ctx.source_text()
                        .get(obj_span.start as usize..obj_span.end as usize)
                })
                .map(|obj_text| {
                    let replacement = format!("Object.getPrototypeOf({obj_text})");
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(member.span.start, member.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                });

            ctx.report(Diagnostic {
                rule_name: "no-proto".to_owned(),
                message:
                    "Use `Object.getPrototypeOf`/`Object.setPrototypeOf` instead of `__proto__`"
                        .to_owned(),
                span: Span::new(member.span.start, member.span.end),
                severity: Severity::Warning,
                help: Some("Replace `.__proto__` with `Object.getPrototypeOf()`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoProto);

    #[test]
    fn test_flags_proto_access() {
        let diags = lint("var p = obj.__proto__;");
        assert_eq!(diags.len(), 1, "__proto__ access should be flagged");
    }

    #[test]
    fn test_allows_get_prototype_of() {
        let diags = lint("var p = Object.getPrototypeOf(obj);");
        assert!(
            diags.is_empty(),
            "Object.getPrototypeOf should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_property() {
        let diags = lint("var x = obj.foo;");
        assert!(
            diags.is_empty(),
            "normal property access should not be flagged"
        );
    }
}
