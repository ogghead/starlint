//! Rule: `jest/prefer-hooks-on-top`
//!
//! Warn when hooks (`beforeEach`, `afterEach`, `beforeAll`, `afterAll`) appear
//! after test cases in a `describe` block. Hooks should be declared before
//! any test cases for readability.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags hooks that appear after `it`/`test` calls in a describe block.
#[derive(Debug)]
pub struct PreferHooksOnTop;

/// Hook function names that should appear before tests.
const HOOK_NAMES: &[&str] = &["beforeAll", "beforeEach", "afterEach", "afterAll"];

/// Test function names.
const TEST_NAMES: &[&str] = &["it", "test"];

impl LintRule for PreferHooksOnTop {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-hooks-on-top".to_owned(),
            description: "Warn when hooks are not at the top of the describe block".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `describe(...)` call
        let is_describe = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "describe"
        );
        if !is_describe {
            return;
        }

        // Get the callback body
        let Some(second_arg_id) = call.arguments.get(1) else {
            return;
        };

        // Resolve the body_id first, then clone statements to release the borrow on ctx.
        let func_body_id = match ctx.node(*second_arg_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => Some(arrow.body),
            Some(AstNode::Function(func)) => func.body,
            _ => return,
        };

        let Some(func_body_id) = func_body_id else {
            return;
        };

        let body_stmts = match ctx.node(func_body_id) {
            Some(AstNode::FunctionBody(body)) => body.statements.clone(),
            _ => return,
        };

        // Scan statements: once we see a test, any subsequent hook is out of place
        let mut seen_test = false;
        for stmt_id in &*body_stmts {
            let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(*stmt_id) else {
                continue;
            };
            let Some(AstNode::CallExpression(inner_call)) = ctx.node(expr_stmt.expression) else {
                continue;
            };
            let callee_name = match ctx.node(inner_call.callee) {
                Some(AstNode::IdentifierReference(id)) => id.name.as_str(),
                _ => continue,
            };

            if TEST_NAMES.contains(&callee_name) {
                seen_test = true;
            } else if seen_test && HOOK_NAMES.contains(&callee_name) {
                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-hooks-on-top".to_owned(),
                    message: format!(
                        "`{callee_name}` should be declared before any test cases in the describe block"
                    ),
                    span: Span::new(inner_call.span.start, inner_call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferHooksOnTop)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_hook_after_test() {
        let source = r"
describe('suite', () => {
    test('first', () => {});
    beforeEach(() => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`beforeEach` after `test` should be flagged"
        );
    }

    #[test]
    fn test_allows_hooks_before_tests() {
        let source = r"
describe('suite', () => {
    beforeEach(() => {});
    afterEach(() => {});
    test('first', () => {});
});
";
        let diags = lint(source);
        assert!(diags.is_empty(), "hooks before tests should not be flagged");
    }

    #[test]
    fn test_flags_after_all_after_test() {
        let source = r"
describe('suite', () => {
    it('works', () => {});
    afterAll(() => {});
});
";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`afterAll` after `it` should be flagged");
    }
}
