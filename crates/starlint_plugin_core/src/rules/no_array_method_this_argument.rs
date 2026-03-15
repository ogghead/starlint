//! Rule: `no-array-method-this-argument`
//!
//! Disallows using the `thisArg` parameter on array iteration methods.
//! Methods like `map`, `filter`, `some`, `every`, `find`, etc. accept
//! an optional second argument that sets `this` inside the callback.
//! Modern JavaScript should use arrow functions (which capture `this`
//! lexically) instead of relying on `thisArg`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Array methods that accept a `thisArg` as their second parameter.
const METHODS_WITH_THIS_ARG: &[&str] = &[
    "every",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flatMap",
    "forEach",
    "map",
    "some",
];

/// Flags array method calls that pass a `thisArg` second argument.
#[derive(Debug)]
pub struct NoArrayMethodThisArgument;

impl LintRule for NoArrayMethodThisArgument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-method-this-argument".to_owned(),
            description: "Disallow using the thisArg parameter on array iteration methods"
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();
        if !METHODS_WITH_THIS_ARG.contains(&method_name) {
            return;
        }

        // These methods accept (callback, thisArg) — flag when more than 1 argument
        if call.arguments.len() <= 1 {
            return;
        }

        // Fix: remove the thisArg (second argument) — delete from end of first arg to end of second
        let fix = call
            .arguments
            .first()
            .zip(call.arguments.get(1))
            .and_then(|(first, second)| {
                let first_span = ctx.node(*first).map(starlint_ast::AstNode::span)?;
                let second_span = ctx.node(*second).map(starlint_ast::AstNode::span)?;
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Remove the `thisArg` parameter".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(first_span.end, second_span.end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                })
            });

        ctx.report(Diagnostic {
            rule_name: "no-array-method-this-argument".to_owned(),
            message: format!(
                "Do not use the `thisArg` parameter of `.{method_name}()` — use an arrow function instead"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoArrayMethodThisArgument);

    #[test]
    fn test_flags_map_with_this_arg() {
        let diags = lint("arr.map(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.map(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_filter_with_this_arg() {
        let diags = lint("arr.filter(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.filter(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_every_with_this_arg() {
        let diags = lint("arr.every(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.every(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_find_with_this_arg() {
        let diags = lint("arr.find(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.find(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_some_with_this_arg() {
        let diags = lint("arr.some(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.some(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_allows_map_without_this_arg() {
        let diags = lint("arr.map(fn);");
        assert!(diags.is_empty(), "arr.map(fn) should not be flagged");
    }

    #[test]
    fn test_allows_reduce_with_initial_value() {
        let diags = lint("arr.reduce(fn, init);");
        assert!(
            diags.is_empty(),
            "arr.reduce(fn, init) should not be flagged (second arg is initial value)"
        );
    }

    #[test]
    fn test_allows_arrow_function_callback() {
        let diags = lint("arr.map(x => x * 2);");
        assert!(
            diags.is_empty(),
            "arrow function callback without thisArg should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("arr.indexOf(value, fromIndex);");
        assert!(
            diags.is_empty(),
            "indexOf with two args should not be flagged"
        );
    }
}
