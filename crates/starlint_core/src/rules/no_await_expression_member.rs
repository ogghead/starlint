//! Rule: `no-await-expression-member`
//!
//! Disallow member access on `await` expressions like `(await foo()).bar`.
//! This pattern is error-prone — prefer assigning to a variable first.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags member expressions on `await` expressions.
#[derive(Debug)]
pub struct NoAwaitExpressionMember;

/// Check if a node (possibly through parenthesized expressions) is an await expression.
/// Since `starlint_ast` doesn't have `ParenthesizedExpression` nodes, we just check directly.
fn is_await_expression(id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.node(id), Some(AstNode::AwaitExpression(_)))
}

impl LintRule for NoAwaitExpressionMember {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-await-expression-member".to_owned(),
            description: "Disallow member access on `await` expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ComputedMemberExpression,
            AstNodeType::StaticMemberExpression,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::StaticMemberExpression(member) => {
                if is_await_expression(member.object, ctx) {
                    ctx.report(Diagnostic {
                        rule_name: "no-await-expression-member".to_owned(),
                        message: "Do not access a member directly on an `await` expression — assign to a variable first".to_owned(),
                        span: Span::new(member.span.start, member.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::ComputedMemberExpression(member) => {
                if is_await_expression(member.object, ctx) {
                    ctx.report(Diagnostic {
                        rule_name: "no-await-expression-member".to_owned(),
                        message: "Do not access a member directly on an `await` expression — assign to a variable first".to_owned(),
                        span: Span::new(member.span.start, member.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAwaitExpressionMember)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_static_member_on_await() {
        let diags = lint("async function f() { (await promise).value; }");
        assert_eq!(diags.len(), 1, "(await promise).value should be flagged");
    }

    #[test]
    fn test_flags_computed_member_on_await() {
        let diags = lint("async function f() { (await promise)[0]; }");
        assert_eq!(diags.len(), 1, "(await promise)[0] should be flagged");
    }

    #[test]
    fn test_allows_variable_then_member() {
        let diags = lint("async function f() { const val = await promise; val.value; }");
        assert!(
            diags.is_empty(),
            "accessing member on a variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_bare_await() {
        let diags = lint("async function f() { await promise; }");
        assert!(diags.is_empty(), "bare await should not be flagged");
    }
}
