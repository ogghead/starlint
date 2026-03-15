//! Rule: `no-async-endpoint-handlers`
//!
//! Flag async functions passed as Express-style route handlers.
//! Unhandled promise rejections in Express crash the server because
//! Express does not catch promise rejections from middleware.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags async functions passed as route handler arguments to Express-style methods.
#[derive(Debug)]
pub struct NoAsyncEndpointHandlers;

/// HTTP method names and Express middleware methods.
const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "use", "all"];

/// Check if an argument node is an async function or async arrow function.
const fn is_async_function_node(node: &AstNode) -> bool {
    match node {
        AstNode::Function(func) => func.is_async,
        AstNode::ArrowFunctionExpression(arrow) => arrow.is_async,
        _ => false,
    }
}

impl LintRule for NoAsyncEndpointHandlers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-async-endpoint-handlers".to_owned(),
            description: "Disallow async functions as Express route handlers".to_owned(),
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

        // Check if callee is a member expression like app.get, router.post, etc.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();
        if !HTTP_METHODS.contains(&method_name) {
            return;
        }

        // Check if any argument is an async function
        for &arg_id in &*call.arguments {
            let Some(arg_node) = ctx.node(arg_id) else {
                continue;
            };
            if is_async_function_node(arg_node) {
                ctx.report(Diagnostic {
                    rule_name: "no-async-endpoint-handlers".to_owned(),
                    message: format!(
                        "Unexpected async function passed to `.{method_name}()` handler — Express does not catch promise rejections"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoAsyncEndpointHandlers);

    #[test]
    fn test_flags_async_arrow_handler() {
        let diags = lint("app.get('/', async (req, res) => {});");
        assert_eq!(diags.len(), 1, "async arrow handler should be flagged");
    }

    #[test]
    fn test_flags_async_function_handler() {
        let diags = lint("router.post('/api', async function(req, res) {});");
        assert_eq!(diags.len(), 1, "async function handler should be flagged");
    }

    #[test]
    fn test_allows_sync_handler() {
        let diags = lint("app.get('/', (req, res) => {});");
        assert!(diags.is_empty(), "sync handler should not be flagged");
    }

    #[test]
    fn test_allows_non_http_method() {
        let diags = lint("app.listen(3000, async () => {});");
        assert!(diags.is_empty(), "non-HTTP method should not be flagged");
    }

    #[test]
    fn test_flags_use_middleware() {
        let diags = lint("app.use(async (req, res, next) => {});");
        assert_eq!(
            diags.len(),
            1,
            "async middleware in .use() should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_function_call() {
        let diags = lint("doSomething(async () => {});");
        assert!(diags.is_empty(), "non-member call should not be flagged");
    }
}
