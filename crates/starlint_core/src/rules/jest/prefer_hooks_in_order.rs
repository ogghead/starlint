//! Rule: `jest/prefer-hooks-in-order`
//!
//! Warn when hooks are not in the standard order: `beforeAll`, `beforeEach`,
//! `afterEach`, `afterAll`. Consistent ordering improves readability and
//! makes the lifecycle flow explicit.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags hooks that are not in the standard lifecycle order.
#[derive(Debug)]
pub struct PreferHooksInOrder;

/// Expected hook order (lower index = should come first).
const HOOK_ORDER: &[&str] = &["beforeAll", "beforeEach", "afterEach", "afterAll"];

impl LintRule for PreferHooksInOrder {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-hooks-in-order".to_owned(),
            description: "Warn when hooks are not in the standard lifecycle order".to_owned(),
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

        // Collect hooks with their order index and span
        let mut last_order: Option<usize> = None;
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

            let Some(order) = HOOK_ORDER.iter().position(|&h| h == callee_name) else {
                continue;
            };

            if let Some(prev_order) = last_order {
                if order < prev_order {
                    ctx.report(Diagnostic {
                        rule_name: "jest/prefer-hooks-in-order".to_owned(),
                        message: format!(
                            "`{callee_name}` should be placed before `{}` in the describe block",
                            HOOK_ORDER.get(prev_order).copied().unwrap_or("unknown")
                        ),
                        span: Span::new(inner_call.span.start, inner_call.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            last_order = Some(order);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferHooksInOrder)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_wrong_order() {
        let source = r"
describe('suite', () => {
    afterEach(() => {});
    beforeEach(() => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`beforeEach` after `afterEach` should be flagged"
        );
    }

    #[test]
    fn test_allows_correct_order() {
        let source = r"
describe('suite', () => {
    beforeAll(() => {});
    beforeEach(() => {});
    afterEach(() => {});
    afterAll(() => {});
});
";
        let diags = lint(source);
        assert!(diags.is_empty(), "correct hook order should not be flagged");
    }

    #[test]
    fn test_flags_before_all_after_after_all() {
        let source = r"
describe('suite', () => {
    afterAll(() => {});
    beforeAll(() => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`beforeAll` after `afterAll` should be flagged"
        );
    }
}
