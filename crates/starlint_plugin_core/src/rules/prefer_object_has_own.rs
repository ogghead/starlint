//! Rule: `prefer-object-has-own`
//!
//! Prefer `Object.hasOwn()` over `Object.prototype.hasOwnProperty.call()`.
//! `Object.hasOwn()` (ES2022) is shorter and more intuitive.

#![allow(clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::fix_utils::source_text_for_span;
use starlint_rule_framework::{FixBuilder, LintContext, LintRule};

/// Flags `Object.prototype.hasOwnProperty.call()` patterns.
#[derive(Debug)]
pub struct PreferObjectHasOwn;

impl LintRule for PreferObjectHasOwn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-object-has-own".to_owned(),
            description: "Prefer `Object.hasOwn()` over `Object.prototype.hasOwnProperty.call()`"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for pattern: <something>.hasOwnProperty.call(...)
        // or Object.prototype.hasOwnProperty.call(...)
        let Some(AstNode::StaticMemberExpression(outer_member)) = ctx.node(call.callee) else {
            return;
        };

        if outer_member.property.as_str() != "call" {
            return;
        }

        // The object should be <something>.hasOwnProperty
        let Some(AstNode::StaticMemberExpression(inner_member)) = ctx.node(outer_member.object)
        else {
            return;
        };

        if inner_member.property.as_str() != "hasOwnProperty" {
            return;
        }

        // Check if it's Object.prototype.hasOwnProperty or {}.hasOwnProperty
        let is_object_prototype = is_object_prototype_pattern(inner_member.object, ctx);
        let is_object_literal = matches!(
            ctx.node(inner_member.object),
            Some(AstNode::ObjectExpression(_))
        );

        if is_object_prototype || is_object_literal {
            let source = ctx.source_text();
            let fix =
                call.arguments
                    .first()
                    .zip(call.arguments.last())
                    .and_then(|(first, last)| {
                        let f_span = ctx.node(*first).map_or(
                            starlint_ast::types::Span::EMPTY,
                            starlint_ast::AstNode::span,
                        );
                        let l_span = ctx.node(*last).map_or(
                            starlint_ast::types::Span::EMPTY,
                            starlint_ast::AstNode::span,
                        );
                        let args_text =
                            source_text_for_span(source, Span::new(f_span.start, l_span.end))
                                .unwrap_or("");
                        FixBuilder::new("Replace with `Object.hasOwn()`", FixKind::SafeFix)
                            .replace(
                                Span::new(call.span.start, call.span.end),
                                format!("Object.hasOwn({args_text})"),
                            )
                            .build()
                    });

            ctx.report(Diagnostic {
                rule_name: "prefer-object-has-own".to_owned(),
                message:
                    "Use `Object.hasOwn()` instead of `Object.prototype.hasOwnProperty.call()`"
                        .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `Object.hasOwn()`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if expression is `Object.prototype`.
fn is_object_prototype_pattern(id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(id) else {
        return false;
    };

    if member.property.as_str() != "prototype" {
        return false;
    }

    matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Object")
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferObjectHasOwn);

    #[test]
    fn test_flags_object_prototype_has_own_property_call() {
        let diags = lint("Object.prototype.hasOwnProperty.call(obj, 'key');");
        assert_eq!(
            diags.len(),
            1,
            "Object.prototype.hasOwnProperty.call() should be flagged"
        );
    }

    #[test]
    fn test_allows_object_has_own() {
        let diags = lint("Object.hasOwn(obj, 'key');");
        assert!(diags.is_empty(), "Object.hasOwn() should not be flagged");
    }

    #[test]
    fn test_allows_direct_has_own_property() {
        let diags = lint("obj.hasOwnProperty('key');");
        assert!(
            diags.is_empty(),
            "direct hasOwnProperty call should not be flagged"
        );
    }
}
