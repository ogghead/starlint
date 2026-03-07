//! Rule: `typescript/await-thenable`
//!
//! Disallow awaiting non-thenable values. Flags `await` expressions where
//! the argument is a literal value (string, number, boolean, null, undefined,
//! array literal, or object literal) that can never be a thenable.
//!
//! Simplified syntax-only version — full checking requires type information.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "typescript/await-thenable";

/// Flags `await` expressions whose argument is a non-thenable literal value.
#[derive(Debug)]
pub struct AwaitThenable;

impl LintRule for AwaitThenable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow awaiting non-thenable values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AwaitExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AwaitExpression(await_expr) = node else {
            return;
        };

        let Some(arg_node) = ctx.node(await_expr.argument) else {
            return;
        };
        if is_non_thenable(arg_node) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Unexpected `await` of a non-thenable value — this has no effect"
                    .to_owned(),
                span: Span::new(await_expr.span.start, await_expr.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if the expression is a literal or structural value that
/// cannot be a thenable: numeric, string, boolean, null, undefined, array
/// literal, or object literal.
fn is_non_thenable(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::NumericLiteral(_)
            | AstNode::StringLiteral(_)
            | AstNode::BooleanLiteral(_)
            | AstNode::NullLiteral(_)
            | AstNode::ArrayExpression(_)
            | AstNode::ObjectExpression(_)
    ) || is_undefined_identifier(node)
}

/// Returns `true` if the expression is the identifier `undefined`.
fn is_undefined_identifier(node: &AstNode) -> bool {
    matches!(node, AstNode::IdentifierReference(ident) if ident.name == "undefined")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AwaitThenable)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_await_string_literal() {
        let diags = lint("async function f() { await \"hello\"; }");
        assert_eq!(diags.len(), 1, "await on string literal should be flagged");
    }

    #[test]
    fn test_flags_await_number_literal() {
        let diags = lint("async function f() { await 42; }");
        assert_eq!(diags.len(), 1, "await on number literal should be flagged");
    }

    #[test]
    fn test_flags_await_boolean_literal() {
        let diags = lint("async function f() { await true; }");
        assert_eq!(diags.len(), 1, "await on boolean literal should be flagged");
    }

    #[test]
    fn test_flags_await_null() {
        let diags = lint("async function f() { await null; }");
        assert_eq!(diags.len(), 1, "await on null should be flagged");
    }

    #[test]
    fn test_flags_await_array_literal() {
        let diags = lint("async function f() { await [1, 2, 3]; }");
        assert_eq!(diags.len(), 1, "await on array literal should be flagged");
    }

    #[test]
    fn test_allows_await_function_call() {
        let diags = lint("async function f() { await fetch('/api'); }");
        assert!(
            diags.is_empty(),
            "await on function call should not be flagged"
        );
    }

    #[test]
    fn test_allows_await_variable() {
        let diags = lint("async function f() { await promise; }");
        assert!(
            diags.is_empty(),
            "await on variable reference should not be flagged"
        );
    }
}
