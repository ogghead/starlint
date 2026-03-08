//! Rule: `jest/expect-expect`
//!
//! Warn when a test has no `expect()` call inside its callback body.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/expect-expect";

/// Flags test blocks (`it`/`test`) that contain no `expect()` calls.
#[derive(Debug)]
pub struct ExpectExpect;

impl LintRule for ExpectExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require at least one `expect()` call in each test".to_owned(),
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

        // Check callee is `it` or `test`
        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str(),
            _ => return,
        };

        if callee_name != "it" && callee_name != "test" {
            return;
        }

        // Get the callback (second argument)
        let Some(callback_id) = call.arguments.get(1) else {
            return;
        };

        // Extract the body span to search for `expect(` in source
        let (body_start, body_end) = match ctx.node(*callback_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => (arrow.span.start, arrow.span.end),
            Some(AstNode::Function(func)) => (func.span.start, func.span.end),
            _ => return,
        };

        let source = ctx.source_text();
        let start = usize::try_from(body_start).unwrap_or(0);
        let end = usize::try_from(body_end).unwrap_or(0);
        let body_source = source.get(start..end).unwrap_or("");

        if !body_source.contains("expect(") {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Test `{callee_name}()` has no `expect()` call — tests should assert something"
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExpectExpect)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_test_without_expect() {
        let diags = lint("test('does nothing', () => { const x = 1; });");
        assert_eq!(diags.len(), 1, "test without expect should be flagged");
    }

    #[test]
    fn test_allows_test_with_expect() {
        let diags = lint("test('works', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "test with expect should not be flagged");
    }

    #[test]
    fn test_flags_it_without_expect() {
        let diags = lint("it('does nothing', () => { console.log('hi'); });");
        assert_eq!(diags.len(), 1, "it() without expect should be flagged");
    }
}
