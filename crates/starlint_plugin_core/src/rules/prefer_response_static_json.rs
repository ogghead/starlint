//! Rule: `prefer-response-static-json`
//!
//! Prefer `Response.json()` over `new Response(JSON.stringify())`.
//! The static `Response.json()` method is cleaner and automatically
//! sets the `Content-Type` header to `application/json`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Response(JSON.stringify(...))` patterns.
#[derive(Debug)]
pub struct PreferResponseStaticJson;

impl LintRule for PreferResponseStaticJson {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-response-static-json".to_owned(),
            description: "Prefer `Response.json()` over `new Response(JSON.stringify())`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        // Check that the constructor is `Response`.
        let Some(AstNode::IdentifierReference(callee_id)) = ctx.node(new_expr.callee) else {
            return;
        };

        if callee_id.name.as_str() != "Response" {
            return;
        }

        // Check the first argument is a call to `JSON.stringify()`.
        let Some(first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        let is_json_stringify = match ctx.node(*first_arg_id) {
            Some(AstNode::CallExpression(call)) => is_json_stringify_call(call.callee, ctx),
            _ => false,
        };

        if is_json_stringify {
            // Extract the argument to JSON.stringify to build the fix
            #[allow(clippy::as_conversions)]
            let fix = if let Some(AstNode::CallExpression(stringify_call)) = ctx.node(*first_arg_id)
            {
                stringify_call.arguments.first().and_then(|inner_arg_id| {
                    let inner_span = ctx.node(*inner_arg_id)?.span();
                    let source = ctx.source_text();
                    let arg_text = source
                        .get(inner_span.start as usize..inner_span.end as usize)?
                        .to_owned();
                    // Check for second argument (options) to new Response
                    let options_text = new_expr.arguments.get(1).and_then(|opts_id| {
                        let opts_span = ctx.node(*opts_id)?.span();
                        source
                            .get(opts_span.start as usize..opts_span.end as usize)
                            .map(ToOwned::to_owned)
                    });
                    let replacement = if let Some(opts) = options_text {
                        format!("Response.json({arg_text}, {opts})")
                    } else {
                        format!("Response.json({arg_text})")
                    };
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(new_expr.span.start, new_expr.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                })
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: "prefer-response-static-json".to_owned(),
                message: "Prefer `Response.json()` over `new Response(JSON.stringify())`"
                    .to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Warning,
                help: Some(
                    "Use `Response.json(data)` — it is cleaner and sets Content-Type automatically"
                        .to_owned(),
                ),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is `JSON.stringify`.
fn is_json_stringify_call(callee_id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.node(callee_id),
        Some(AstNode::StaticMemberExpression(member))
            if member.property.as_str() == "stringify"
            && matches!(
                ctx.node(member.object),
                Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "JSON"
            )
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferResponseStaticJson);

    #[test]
    fn test_flags_new_response_json_stringify() {
        let diags = lint("new Response(JSON.stringify(data));");
        assert_eq!(
            diags.len(),
            1,
            "new Response(JSON.stringify()) should be flagged"
        );
    }

    #[test]
    fn test_flags_with_options() {
        let diags = lint("new Response(JSON.stringify(data), { status: 200 });");
        assert_eq!(
            diags.len(),
            1,
            "new Response(JSON.stringify(), options) should be flagged"
        );
    }

    #[test]
    fn test_allows_plain_string() {
        let diags = lint("new Response('hello');");
        assert!(
            diags.is_empty(),
            "new Response with a plain string should not be flagged"
        );
    }

    #[test]
    fn test_allows_response_json() {
        let diags = lint("Response.json(data);");
        assert!(diags.is_empty(), "Response.json() should not be flagged");
    }

    #[test]
    fn test_allows_new_response_no_args() {
        let diags = lint("new Response();");
        assert!(
            diags.is_empty(),
            "new Response() with no args should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_constructor() {
        let diags = lint("new Foo(JSON.stringify(data));");
        assert!(
            diags.is_empty(),
            "non-Response constructor should not be flagged"
        );
    }
}
