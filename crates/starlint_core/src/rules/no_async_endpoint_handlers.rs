//! Rule: `no-async-endpoint-handlers`
//!
//! Flag async functions passed as Express-style route handlers.
//! Unhandled promise rejections in Express crash the server because
//! Express does not catch promise rejections from middleware.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags async functions passed as route handler arguments to Express-style methods.
#[derive(Debug)]
pub struct NoAsyncEndpointHandlers;

/// HTTP method names and Express middleware methods.
const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "use", "all"];

/// Check if an argument is an async function or async arrow function.
fn is_async_function_arg(arg: &Argument<'_>) -> bool {
    match arg {
        Argument::FunctionExpression(func) => func.r#async,
        Argument::ArrowFunctionExpression(arrow) => arrow.r#async,
        _ => false,
    }
}

impl NativeRule for NoAsyncEndpointHandlers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-async-endpoint-handlers".to_owned(),
            description: "Disallow async functions as Express route handlers".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if callee is a member expression like app.get, router.post, etc.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();
        if !HTTP_METHODS.contains(&method_name) {
            return;
        }

        // Check if any argument is an async function
        for arg in &call.arguments {
            if is_async_function_arg(arg) {
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAsyncEndpointHandlers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

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
