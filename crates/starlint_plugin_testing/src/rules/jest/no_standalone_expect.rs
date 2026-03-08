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
    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("expect(")
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

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
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

        // Check if this expect is inside a test/it callback
        let source = ctx.source_text();
        let pos = usize::try_from(call.span.start).unwrap_or(0);
        let before = source.get(..pos).unwrap_or("");

        if !is_inside_test_callback(before) {
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

/// Check if a position is inside a test/it callback by finding the last
/// `test(`/`it(` call and counting brace depth.
fn is_inside_test_callback(before: &str) -> bool {
    let last_test = before.rfind("test(");
    let last_it = before.rfind("it(");

    // Also consider beforeEach/afterEach/beforeAll/afterAll as valid containers
    let last_before_each = before.rfind("beforeEach(");
    let last_after_each = before.rfind("afterEach(");
    let last_before_all = before.rfind("beforeAll(");
    let last_after_all = before.rfind("afterAll(");

    let call_pos = [
        last_test,
        last_it,
        last_before_each,
        last_after_each,
        last_before_all,
        last_after_all,
    ]
    .into_iter()
    .flatten()
    .max();

    let Some(pos) = call_pos else {
        return false;
    };

    let after_call = before.get(pos..).unwrap_or("");
    let mut brace_depth: i32 = 0;
    for ch in after_call.chars() {
        if ch == '{' {
            brace_depth = brace_depth.saturating_add(1);
        } else if ch == '}' {
            brace_depth = brace_depth.saturating_sub(1);
        }
    }

    brace_depth > 0
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoStandaloneExpect)];
        lint_source(source, "test.js", &rules)
    }

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
