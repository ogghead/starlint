//! Rule: `jest/no-done-callback`
//!
//! Warn when a `done` callback parameter is used in test/hook callbacks.
//! Prefer async/await patterns instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-done-callback";

/// Test and hook function names to check.
const CALLBACK_FUNS: &[&str] = &[
    "it",
    "test",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
];

/// Flags test/hook callbacks that use a `done` parameter.
#[derive(Debug)]
pub struct NoDoneCallback;

impl LintRule for NoDoneCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `done` callback in tests -- use async/await instead".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("done") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check callee is a test/hook function
        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.clone(),
            _ => return,
        };

        if !CALLBACK_FUNS.contains(&callee_name.as_str()) {
            return;
        }

        // For it/test, callback is the second arg; for hooks, it's the first
        let callback_idx = usize::from(callee_name == "it" || callee_name == "test");

        let Some(callback_id) = call.arguments.get(callback_idx) else {
            return;
        };

        // Check if the callback has a parameter named `done`
        let has_done = match ctx.node(*callback_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => {
                arrow.params.iter().any(|p| {
                    matches!(ctx.node(*p), Some(AstNode::BindingIdentifier(id)) if id.name.as_str() == "done")
                })
            }
            Some(AstNode::Function(func)) => {
                func.params.iter().any(|p| {
                    matches!(ctx.node(*p), Some(AstNode::BindingIdentifier(id)) if id.name.as_str() == "done")
                })
            }
            _ => false,
        };

        if has_done {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Avoid using a `done` callback in `{callee_name}()` -- use async/await instead"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoDoneCallback);

    #[test]
    fn test_flags_done_in_test() {
        let diags = lint("test('async', (done) => { done(); });");
        assert_eq!(diags.len(), 1, "done callback in test should be flagged");
    }

    #[test]
    fn test_flags_done_in_before_each() {
        let diags = lint("beforeEach((done) => { done(); });");
        assert_eq!(
            diags.len(),
            1,
            "done callback in beforeEach should be flagged"
        );
    }

    #[test]
    fn test_allows_async_test() {
        let diags = lint("test('async', async () => { await something(); });");
        assert!(
            diags.is_empty(),
            "async test without done should not be flagged"
        );
    }
}
