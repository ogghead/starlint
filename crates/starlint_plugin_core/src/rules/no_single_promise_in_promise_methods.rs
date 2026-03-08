//! Rule: `no-single-promise-in-promise-methods`
//!
//! Disallow passing single-element arrays to `Promise.all()`, `Promise.race()`,
//! `Promise.allSettled()`, and `Promise.any()`. These methods are designed to
//! operate on multiple promises — passing a single-element array is likely a
//! mistake and should be replaced with the promise itself or `Promise.resolve()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Promise methods that expect multiple promises.
const PROMISE_AGGREGATE_METHODS: &[&str] = &["all", "race", "allSettled", "any"];

/// Flags `Promise.all([x])`, `Promise.race([x])`, etc. with a single element.
#[derive(Debug)]
pub struct NoSinglePromiseInPromiseMethods;

impl LintRule for NoSinglePromiseInPromiseMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-single-promise-in-promise-methods".to_owned(),
            description: "Disallow passing single-element arrays to Promise aggregate methods"
                .to_owned(),
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

        // Check for `Promise.<method>(...)` pattern via static member access.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        // Object must be `Promise`.
        if !matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(ident)) if ident.name == "Promise")
        {
            return;
        }

        let method_name = member.property.as_str();
        if !PROMISE_AGGREGATE_METHODS.contains(&method_name) {
            return;
        }

        // Must have exactly one argument, and it must be an array expression.
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        let Some(AstNode::ArrayExpression(array)) = ctx.node(first_arg_id) else {
            return;
        };

        // Flag if the array has exactly one element (and no spread).
        #[allow(clippy::as_conversions)] // u32→usize is lossless
        if array.elements.len() == 1 {
            // Extract the single element text for the fix
            let fix = array.elements.first().and_then(|&elem_id| {
                let elem_span = ctx.node(elem_id)?.span();
                let source = ctx.source_text();
                let elem_text = source
                    .get(elem_span.start as usize..elem_span.end as usize)
                    .unwrap_or("")
                    .to_owned();
                (!elem_text.is_empty()).then(|| Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Unwrap single-element array".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement: elem_text,
                    }],
                    is_snippet: false,
                })
            });

            ctx.report(Diagnostic {
                rule_name: "no-single-promise-in-promise-methods".to_owned(),
                message: format!(
                    "Unnecessary single-element array in `Promise.{method_name}()` — use the value directly"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Pass the value directly instead of wrapping in an array".to_owned()),
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoSinglePromiseInPromiseMethods)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_promise_all_single() {
        let diags = lint("Promise.all([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.all with single element should be flagged"
        );
    }

    #[test]
    fn test_flags_promise_race_single() {
        let diags = lint("Promise.race([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.race with single element should be flagged"
        );
    }

    #[test]
    fn test_flags_promise_all_settled_single() {
        let diags = lint("Promise.allSettled([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.allSettled with single element should be flagged"
        );
    }

    #[test]
    fn test_flags_promise_any_single() {
        let diags = lint("Promise.any([p1])");
        assert_eq!(
            diags.len(),
            1,
            "Promise.any with single element should be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all_multiple() {
        let diags = lint("Promise.all([p1, p2])");
        assert!(
            diags.is_empty(),
            "Promise.all with multiple elements should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_resolve() {
        let diags = lint("Promise.resolve(p1)");
        assert!(diags.is_empty(), "Promise.resolve should not be flagged");
    }

    #[test]
    fn test_allows_promise_all_non_array() {
        let diags = lint("Promise.all(promises)");
        assert!(
            diags.is_empty(),
            "Promise.all with non-array argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_promise_all_empty_array() {
        let diags = lint("Promise.all([])");
        assert!(
            diags.is_empty(),
            "Promise.all with empty array should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_promise_callee() {
        let diags = lint("MyLib.all([p1])");
        assert!(diags.is_empty(), "non-Promise callee should not be flagged");
    }
}
