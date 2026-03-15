//! Rule: `jest/no-hooks`
//!
//! Warn when lifecycle hooks (`beforeEach`, `afterEach`, `beforeAll`, `afterAll`) are used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-hooks";

/// Hook names that this rule flags.
const HOOK_NAMES: &[&str] = &["beforeEach", "afterEach", "beforeAll", "afterAll"];

/// Flags usage of Jest lifecycle hooks.
#[derive(Debug)]
pub struct NoHooks;

impl LintRule for NoHooks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow usage of Jest lifecycle hooks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("beforeEach")
            || source_text.contains("afterEach")
            || source_text.contains("beforeAll")
            || source_text.contains("afterAll"))
            && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.clone(),
            _ => return,
        };

        if HOOK_NAMES.contains(&callee_name.as_str()) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Unexpected use of `{callee_name}` hook — prefer explicit setup in each test"
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

    starlint_rule_framework::lint_rule_test!(NoHooks);

    #[test]
    fn test_flags_before_each() {
        let diags = lint("beforeEach(() => { setup(); });");
        assert_eq!(diags.len(), 1, "`beforeEach` should be flagged");
    }

    #[test]
    fn test_flags_after_all() {
        let diags = lint("afterAll(() => { cleanup(); });");
        assert_eq!(diags.len(), 1, "`afterAll` should be flagged");
    }

    #[test]
    fn test_allows_regular_calls() {
        let diags = lint("test('works', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "regular test calls should not be flagged");
    }
}
