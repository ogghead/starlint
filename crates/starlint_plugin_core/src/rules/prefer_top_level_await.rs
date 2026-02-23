//! Rule: `prefer-top-level-await`
//!
//! Flag immediately-invoked async functions (async IIFEs) at the top level
//! that could use top-level `await` instead. Patterns like
//! `(async () => { await foo(); })()` can be simplified to just `await foo()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags async IIFEs that could use top-level `await`.
#[derive(Debug)]
pub struct PreferTopLevelAwait;

/// Check if an expression is an async function expression or async arrow.
/// Note: `ParenthesizedExpression` is not represented in `starlint_ast`, so
/// the parser flattens it away. We just check the callee directly.
fn is_async_iife_callee(id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(id) {
        Some(AstNode::Function(func)) => func.is_async,
        Some(AstNode::ArrowFunctionExpression(arrow)) => arrow.is_async,
        _ => false,
    }
}

impl LintRule for PreferTopLevelAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-top-level-await".to_owned(),
            description: "Prefer top-level `await` over async IIFEs".to_owned(),
            category: Category::Suggestion,
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

        // Check if the callee (unwrapping parentheses) is an async function
        // expression or async arrow function expression
        if !is_async_iife_callee(call.callee, ctx) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "prefer-top-level-await".to_owned(),
            message: "Prefer top-level `await` over an immediately-invoked async function"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferTopLevelAwait)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_async_function_iife() {
        let diags = lint("(async function() { await foo(); })();");
        assert_eq!(diags.len(), 1, "async function IIFE should be flagged");
    }

    #[test]
    fn test_flags_async_arrow_iife() {
        let diags = lint("(async () => { await foo(); })();");
        assert_eq!(diags.len(), 1, "async arrow IIFE should be flagged");
    }

    #[test]
    fn test_allows_async_function_declaration() {
        let diags = lint("async function foo() { await bar(); }");
        assert!(
            diags.is_empty(),
            "async function declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_sync_iife() {
        let diags = lint("(function() { })();");
        assert!(diags.is_empty(), "sync IIFE should not be flagged");
    }

    #[test]
    fn test_allows_sync_arrow_iife() {
        let diags = lint("(() => { })();");
        assert!(diags.is_empty(), "sync arrow IIFE should not be flagged");
    }

    #[test]
    fn test_allows_normal_async_call() {
        let diags = lint("doSomething();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
