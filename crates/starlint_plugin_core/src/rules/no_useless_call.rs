//! Rule: `no-useless-call`
//!
//! Disallow unnecessary `.call()` and `.apply()`. Using `foo.call(thisArg)`
//! when `thisArg` is the receiver is equivalent to just `foo()` and the
//! `.call()`/`.apply()` is unnecessary.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags unnecessary `.call()` and `.apply()` invocations.
#[derive(Debug)]
pub struct NoUselessCall;

impl LintRule for NoUselessCall {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-call".to_owned(),
            description: "Disallow unnecessary `.call()` and `.apply()`".to_owned(),
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

        let method = member.property.as_str();
        if method != "call" && method != "apply" {
            return;
        }

        // Must have at least one argument (thisArg)
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        // Check if thisArg is `null` or `undefined` — this means the function
        // is called without a specific this binding, which is useless
        let is_null_or_undefined = match ctx.node(first_arg_id) {
            Some(AstNode::NullLiteral(_)) => true,
            Some(AstNode::IdentifierReference(id)) => id.name.as_str() == "undefined",
            _ => false,
        };

        if is_null_or_undefined {
            // Build fix: foo.call(null, a, b) → foo(a, b)
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let Some(obj_node) = ctx.node(member.object) else {
                    return;
                };
                let obj_span = obj_node.span();
                let obj_text = source
                    .get(obj_span.start as usize..obj_span.end as usize)
                    .unwrap_or("");

                // Collect remaining args (skip thisArg)
                let remaining_args: Vec<&str> = call
                    .arguments
                    .iter()
                    .skip(1)
                    .filter_map(|&arg_id| {
                        let s = ctx.node(arg_id)?.span();
                        source.get(s.start as usize..s.end as usize)
                    })
                    .collect();

                let args_str = remaining_args.join(", ");
                let replacement = format!("{obj_text}({args_str})");
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                })
            };

            ctx.report(Diagnostic {
                rule_name: "no-useless-call".to_owned(),
                message: format!("Unnecessary `.{method}()` — call the function directly instead"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Remove `.call()`/`.apply()` and call the function directly".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUselessCall);

    #[test]
    fn test_flags_call_with_null() {
        let diags = lint("foo.call(null, 1, 2);");
        assert_eq!(diags.len(), 1, "foo.call(null, ...) should be flagged");
    }

    #[test]
    fn test_flags_apply_with_undefined() {
        let diags = lint("foo.apply(undefined, [1, 2]);");
        assert_eq!(
            diags.len(),
            1,
            "foo.apply(undefined, ...) should be flagged"
        );
    }

    #[test]
    fn test_allows_call_with_this_arg() {
        let diags = lint("foo.call(obj, 1, 2);");
        assert!(diags.is_empty(), "foo.call(obj, ...) should not be flagged");
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("foo(1, 2);");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
