//! Rule: `no-await-in-promise-methods`
//!
//! Disallow `await` inside `Promise.all()`, `Promise.race()`,
//! `Promise.allSettled()`, and `Promise.any()` array arguments.
//!
//! When you `await` inside the array passed to these methods, the promises
//! are resolved sequentially instead of in parallel, defeating the purpose
//! of using `Promise.all` and friends.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Promise methods that accept an iterable of promises for parallel resolution.
const PROMISE_METHODS: &[&str] = &["all", "race", "allSettled", "any"];

/// Flags `await` expressions inside array arguments to `Promise.all()`,
/// `Promise.race()`, `Promise.allSettled()`, and `Promise.any()`.
#[derive(Debug)]
pub struct NoAwaitInPromiseMethods;

impl LintRule for NoAwaitInPromiseMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-await-in-promise-methods".to_owned(),
            description: "Disallow `await` in Promise.all/race/allSettled/any array arguments"
                .to_owned(),
            category: Category::Correctness,
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

        // Check if callee is `Promise.<method>`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let is_promise = matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(obj)) if obj.name.as_str() == "Promise"
        );

        if !is_promise {
            return;
        }

        let method_name = member.property.as_str();
        if !PROMISE_METHODS.contains(&method_name) {
            return;
        }

        // Check the first argument — should be an array expression
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        let Some(AstNode::ArrayExpression(array)) = ctx.node(first_arg_id) else {
            return;
        };

        // Check if any element in the array is an `await` expression
        // Collect fix edits: for each `await expr`, remove the `await ` prefix
        let mut has_await = false;
        let mut edits: Vec<Edit> = Vec::new();
        for &element_id in &*array.elements {
            if let Some(AstNode::AwaitExpression(await_expr)) = ctx.node(element_id) {
                has_await = true;
                // Remove the `await ` keyword — replace await_expr span with just the argument
                let arg_ast_span = ctx
                    .node(await_expr.argument)
                    .map(starlint_ast::AstNode::span);
                let Some(arg_span_val) = arg_ast_span else {
                    continue;
                };
                let arg_span = Span::new(arg_span_val.start, arg_span_val.end);
                edits.push(Edit {
                    span: Span::new(await_expr.span.start, arg_span.start),
                    replacement: String::new(),
                });
            }
        }

        if has_await {
            let fix = (!edits.is_empty()).then(|| Fix {
                kind: FixKind::SuggestionFix,
                message: "Remove `await` from array elements".to_owned(),
                edits,
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "no-await-in-promise-methods".to_owned(),
                message: format!(
                    "Avoid using `await` inside `Promise.{method_name}()` — it defeats parallel execution"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoAwaitInPromiseMethods);

    #[test]
    fn test_flags_await_in_promise_all() {
        let diags = lint("async function f() { await Promise.all([await p1, await p2]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.all array should be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all_without_await() {
        let diags = lint("async function f() { await Promise.all([p1, p2]); }");
        assert!(
            diags.is_empty(),
            "Promise.all without inner await should not be flagged"
        );
    }

    #[test]
    fn test_flags_await_in_promise_race() {
        let diags = lint("async function f() { await Promise.race([await p1]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.race array should be flagged"
        );
    }

    #[test]
    fn test_flags_await_in_promise_all_settled() {
        let diags = lint("async function f() { await Promise.allSettled([await p1]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.allSettled array should be flagged"
        );
    }

    #[test]
    fn test_flags_await_in_promise_any() {
        let diags = lint("async function f() { await Promise.any([await p1]); }");
        assert_eq!(
            diags.len(),
            1,
            "await inside Promise.any array should be flagged"
        );
    }

    #[test]
    fn test_allows_standalone_await() {
        let diags = lint("async function f() { await p1; }");
        assert!(diags.is_empty(), "standalone await should not be flagged");
    }

    #[test]
    fn test_allows_non_promise_call() {
        let diags = lint("async function f() { Foo.all([await p1]); }");
        assert!(diags.is_empty(), "non-Promise call should not be flagged");
    }
}
