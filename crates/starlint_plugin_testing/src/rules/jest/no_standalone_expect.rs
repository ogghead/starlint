//! Rule: `jest/no-standalone-expect`
//!
//! Error when `expect()` is used outside of `it`/`test` callbacks.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-standalone-expect";

/// Flags `expect()` calls that appear outside of test/it callbacks.
#[derive(Debug)]
pub struct NoStandaloneExpect;

impl LintRule for NoStandaloneExpect {
    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("expect(") && crate::is_test_file(file_path)
    }

    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `expect()` outside of `it`/`test` blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check callee is `expect`
        let is_expect = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect"
        );

        if !is_expect {
            return;
        }

        // Walk up the AST to check if inside a test/hook callback
        if !is_inside_test_via_ancestors(node_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`expect()` must be called inside an `it()` or `test()` block".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Test/hook function names that are valid containers for `expect()`.
const TEST_CALLBACK_NAMES: &[&str] = &[
    "test",
    "it",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
];

/// Walk up the AST parent chain to check if `node_id` is inside a
/// test/hook callback. `O(depth)` instead of `O(source_length)`.
fn is_inside_test_via_ancestors(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let tree = ctx.tree();
    let mut current = tree.parent(node_id);
    while let Some(pid) = current {
        if let Some(AstNode::CallExpression(call)) = tree.get(pid) {
            if let Some(AstNode::IdentifierReference(id)) = tree.get(call.callee) {
                if TEST_CALLBACK_NAMES.contains(&id.name.as_str()) {
                    return true;
                }
            }
        }
        current = tree.parent(pid);
    }
    false
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoStandaloneExpect);

    #[test]
    fn test_flags_standalone_expect() {
        let diags = lint("expect(true).toBe(true);");
        assert_eq!(diags.len(), 1, "standalone expect should be flagged");
    }

    #[test]
    fn test_allows_expect_in_test() {
        let diags = lint("test('ok', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "expect inside test should not be flagged");
    }

    #[test]
    fn test_allows_expect_in_it() {
        let diags = lint("it('ok', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "expect inside it should not be flagged");
    }
}
