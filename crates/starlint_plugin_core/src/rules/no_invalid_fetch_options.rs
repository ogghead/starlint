//! Rule: `no-invalid-fetch-options`
//!
//! Flag `fetch()` calls where the options object contains both a `body`
//! property and a `method` set to `"GET"` or `"HEAD"`. GET and HEAD
//! requests do not accept a body — including one is a bug that most
//! runtimes will either silently ignore or reject.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::PropertyKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `fetch()` calls with `body` on GET/HEAD requests.
#[derive(Debug)]
pub struct NoInvalidFetchOptions;

/// HTTP methods that do not accept a request body.
const BODYLESS_METHODS: &[&str] = &["GET", "HEAD"];

impl LintRule for NoInvalidFetchOptions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-invalid-fetch-options".to_owned(),
            description: "Disallow `body` in `fetch()` options for GET/HEAD requests".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `fetch(...)` call
        if !is_fetch_call(call.callee, ctx) {
            return;
        }

        // Need at least two arguments: url and options
        let Some(second_arg_id) = call.arguments.get(1) else {
            return;
        };

        // Options must be an object literal
        let Some(AstNode::ObjectExpression(options)) = ctx.node(*second_arg_id) else {
            return;
        };

        let mut has_body = false;
        let mut method_value: Option<String> = None;

        for prop_id in &options.properties {
            let Some(AstNode::ObjectProperty(prop)) = ctx.node(*prop_id) else {
                continue;
            };

            // Only check init properties (not getters/setters)
            if prop.kind != PropertyKind::Init {
                continue;
            }

            let Some(key_name) = static_key_name(prop.key, ctx) else {
                continue;
            };

            if key_name == "body" {
                has_body = true;
            } else if key_name == "method" {
                method_value = string_literal_value(prop.value, ctx);
            }
        }

        if has_body {
            if let Some(method) = method_value {
                let upper_method = method.to_uppercase();
                if BODYLESS_METHODS.contains(&upper_method.as_str()) {
                    ctx.report(Diagnostic {
                        rule_name: "no-invalid-fetch-options".to_owned(),
                        message: format!(
                            "`fetch()` with method `{method}` should not have a `body` — {upper_method} requests do not accept a body"
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
        }
    }
}

/// Check if a call expression's callee is `fetch`.
fn is_fetch_call(callee_id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.node(callee_id),
        Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "fetch"
    )
}

/// Extract a static key name from a property key node (identifier or string literal).
fn static_key_name(key_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let node = ctx.node(key_id)?;
    match node {
        AstNode::IdentifierReference(ident) => Some(ident.name.clone()),
        AstNode::BindingIdentifier(ident) => Some(ident.name.clone()),
        AstNode::StringLiteral(lit) => Some(lit.value.clone()),
        _ => None,
    }
}

/// Extract the value of a string literal expression.
fn string_literal_value(id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    if let Some(AstNode::StringLiteral(lit)) = ctx.node(id) {
        Some(lit.value.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInvalidFetchOptions)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_get_with_body() {
        let diags = lint("fetch(url, { method: 'GET', body: data });");
        assert_eq!(diags.len(), 1, "GET with body should be flagged");
    }

    #[test]
    fn test_flags_head_with_body() {
        let diags = lint("fetch(url, { method: 'HEAD', body: data });");
        assert_eq!(diags.len(), 1, "HEAD with body should be flagged");
    }

    #[test]
    fn test_flags_get_lowercase_with_body() {
        let diags = lint("fetch(url, { method: 'get', body: data });");
        assert_eq!(diags.len(), 1, "lowercase get with body should be flagged");
    }

    #[test]
    fn test_allows_post_with_body() {
        let diags = lint("fetch(url, { method: 'POST', body: data });");
        assert!(diags.is_empty(), "POST with body should not be flagged");
    }

    #[test]
    fn test_allows_put_with_body() {
        let diags = lint("fetch(url, { method: 'PUT', body: data });");
        assert!(diags.is_empty(), "PUT with body should not be flagged");
    }

    #[test]
    fn test_allows_fetch_no_options() {
        let diags = lint("fetch(url);");
        assert!(
            diags.is_empty(),
            "fetch with no options should not be flagged"
        );
    }

    #[test]
    fn test_allows_get_no_body() {
        let diags = lint("fetch(url, { method: 'GET' });");
        assert!(diags.is_empty(), "GET without body should not be flagged");
    }

    #[test]
    fn test_allows_body_no_method() {
        // Default method is GET, but without an explicit method property
        // we don't flag it — the user may have a reason, and we only
        // flag the explicit mismatch.
        let diags = lint("fetch(url, { body: data });");
        assert!(
            diags.is_empty(),
            "body without explicit method should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_fetch_call() {
        let diags = lint("request(url, { method: 'GET', body: data });");
        assert!(diags.is_empty(), "non-fetch calls should not be flagged");
    }

    #[test]
    fn test_allows_variable_options() {
        let diags = lint("fetch(url, options);");
        assert!(diags.is_empty(), "variable options should not be flagged");
    }
}
