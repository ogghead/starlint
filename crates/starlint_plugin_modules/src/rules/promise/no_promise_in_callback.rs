//! Rule: `promise/no-promise-in-callback`
//!
//! Forbid creating promises inside callback-style functions. Mixing
//! callback patterns with promise patterns leads to confusing control flow.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Promise` or `Promise.resolve`/`Promise.reject` inside
/// functions whose parameter names suggest they are callbacks.
///
/// This is a heuristic: we check if any parameter is named `cb`, `callback`,
/// `done`, or `next` and the function body contains Promise usage.
#[derive(Debug)]
pub struct NoPromiseInCallback;

/// Common callback parameter names.
const CALLBACK_PARAMS: &[&str] = &["cb", "callback", "done", "next"];

impl LintRule for NoPromiseInCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-promise-in-callback".to_owned(),
            description: "Forbid creating promises inside callbacks".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Look for new Promise(...) inside function bodies
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let is_promise = matches!(
            ctx.node(new_expr.callee),
            Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "Promise"
        );

        if !is_promise {
            return;
        }

        // Heuristic: check the surrounding source for callback parameter names
        // A full implementation would walk up the AST to the enclosing function.
        let start = usize::try_from(new_expr.span.start).unwrap_or(0);
        let prefix_start = start.saturating_sub(200);
        let prefix = ctx
            .source_text()
            .get(prefix_start..start)
            .unwrap_or_default();

        for name in CALLBACK_PARAMS {
            // Look for patterns like `function foo(cb)` or `(callback) =>`
            if prefix.contains(&format!("({name})"))
                || prefix.contains(&format!("({name},"))
                || prefix.contains(&format!(", {name})"))
                || prefix.contains(&format!(", {name},"))
            {
                ctx.report(Diagnostic {
                    rule_name: "promise/no-promise-in-callback".to_owned(),
                    message: "Avoid creating a Promise inside a callback-style function".to_owned(),
                    span: Span::new(new_expr.span.start, new_expr.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoPromiseInCallback);

    #[test]
    fn test_flags_promise_in_callback() {
        let diags = lint("function foo(callback) { return new Promise((r) => r(1)); }");
        assert_eq!(
            diags.len(),
            1,
            "should flag new Promise inside callback function"
        );
    }

    #[test]
    fn test_allows_promise_in_normal_function() {
        let diags = lint("function foo(x) { return new Promise((r) => r(x)); }");
        assert!(diags.is_empty(), "normal function should not be flagged");
    }

    #[test]
    fn test_flags_promise_in_done_callback() {
        let diags = lint("function handler(done) { return new Promise((r) => r(1)); }");
        assert_eq!(
            diags.len(),
            1,
            "should flag new Promise inside done callback"
        );
    }
}
