//! Rule: `prefer-spread`
//!
//! Require spread operator instead of `.apply()`. `foo.apply(null, args)`
//! should be written as `foo(...args)`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.apply()` calls that could use spread syntax.
#[derive(Debug)]
pub struct PreferSpread;

impl LintRule for PreferSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-spread".to_owned(),
            description: "Require spread operator instead of `.apply()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "apply" {
            return;
        }

        // Only autofix `fn.apply(null, args)` or `fn.apply(undefined, args)` patterns
        let source = ctx.source_text();
        let Some(obj_node) = ctx.node(member.object) else {
            return;
        };
        let obj_span = obj_node.span();
        #[allow(clippy::as_conversions)]
        let fn_text = source
            .get(obj_span.start as usize..obj_span.end as usize)
            .unwrap_or("");

        // Try to extract autofix for the 2-arg pattern: fn.apply(null/undefined, args)
        let fix = if call.arguments.len() == 2 {
            let first_arg_id = call.arguments.first();
            let second_arg_id = call.arguments.get(1);
            #[allow(clippy::as_conversions)]
            let is_null_or_undefined = first_arg_id.is_some_and(|a_id| {
                if let Some(a_node) = ctx.node(*a_id) {
                    let a_span = a_node.span();
                    let text = source
                        .get(a_span.start as usize..a_span.end as usize)
                        .unwrap_or("");
                    text == "null" || text == "undefined"
                } else {
                    false
                }
            });
            if is_null_or_undefined {
                #[allow(clippy::as_conversions)]
                second_arg_id.and_then(|args_id| {
                    let args_node = ctx.node(*args_id)?;
                    let args_span = args_node.span();
                    let args_text = source.get(args_span.start as usize..args_span.end as usize)?;
                    let replacement = format!("{fn_text}(...{args_text})");
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                })
            } else {
                None
            }
        } else {
            None
        };

        let message = "Use the spread operator instead of `.apply()`".to_owned();
        ctx.report(Diagnostic {
            rule_name: "prefer-spread".to_owned(),
            message: message.clone(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(message),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferSpread);

    #[test]
    fn test_flags_apply() {
        let diags = lint("foo.apply(null, args);");
        assert_eq!(diags.len(), 1, ".apply() should be flagged");
    }

    #[test]
    fn test_allows_spread() {
        let diags = lint("foo(...args);");
        assert!(diags.is_empty(), "spread operator should not be flagged");
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("foo(1, 2);");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }
}
