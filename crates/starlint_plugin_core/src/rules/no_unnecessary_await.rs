//! Rule: `no-unnecessary-await`
//!
//! Disallow awaiting non-promise values (non-thenables). Awaiting a literal
//! like `await 1`, `await "str"`, `await true`, `await null`, or
//! `await undefined` is pointless — the value is not a thenable and the
//! `await` adds an unnecessary microtask tick.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `await` expressions whose argument is a non-thenable literal.
#[derive(Debug)]
pub struct NoUnnecessaryAwait;

impl LintRule for NoUnnecessaryAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unnecessary-await".to_owned(),
            description: "Disallow awaiting non-promise values".to_owned(),
            category: Category::Suggestion,
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
        if is_non_thenable_literal(arg_node) {
            let await_span = Span::new(await_expr.span.start, await_expr.span.end);
            let arg_node_span = arg_node.span();
            let arg_span = Span::new(arg_node_span.start, arg_node_span.end);
            let arg_text = ctx
                .source_text()
                .get(
                    usize::try_from(arg_span.start).unwrap_or(0)
                        ..usize::try_from(arg_span.end).unwrap_or(0),
                )
                .unwrap_or("")
                .to_owned();
            ctx.report(Diagnostic {
                rule_name: "no-unnecessary-await".to_owned(),
                message: "Unnecessary `await` on a non-thenable value".to_owned(),
                span: await_span,
                severity: Severity::Warning,
                help: Some("Remove the `await` keyword".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove `await`".to_owned(),
                    edits: vec![Edit {
                        span: await_span,
                        replacement: arg_text,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if the node is a literal that cannot be a thenable:
/// numeric, string, boolean, null, or the identifier `undefined`.
fn is_non_thenable_literal(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::NumericLiteral(_)
            | AstNode::StringLiteral(_)
            | AstNode::BooleanLiteral(_)
            | AstNode::NullLiteral(_)
    ) || is_undefined_identifier(node)
}

/// Returns `true` if the node is an identifier named `undefined`.
fn is_undefined_identifier(node: &AstNode) -> bool {
    matches!(node, AstNode::IdentifierReference(ident) if ident.name == "undefined")
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnnecessaryAwait);

    #[test]
    fn test_flags_await_numeric_literal() {
        let diags = lint("async function f() { await 1; }");
        assert_eq!(diags.len(), 1, "await on numeric literal should be flagged");
    }

    #[test]
    fn test_flags_await_string_literal() {
        let diags = lint("async function f() { await \"hello\"; }");
        assert_eq!(diags.len(), 1, "await on string literal should be flagged");
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
    fn test_flags_await_undefined() {
        let diags = lint("async function f() { await undefined; }");
        assert_eq!(diags.len(), 1, "await on undefined should be flagged");
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

    #[test]
    fn test_allows_await_new_promise() {
        let diags = lint("async function f() { await new Promise(resolve => resolve()); }");
        assert!(
            diags.is_empty(),
            "await on new Promise should not be flagged"
        );
    }
}
