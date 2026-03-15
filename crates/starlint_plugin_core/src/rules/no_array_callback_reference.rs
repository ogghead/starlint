//! Rule: `no-array-callback-reference`
//!
//! Disallows passing function references directly to array iteration methods.
//! When a function reference is passed (e.g. `arr.map(parseInt)`), the
//! iteration method passes extra arguments (`index`, `array`) that the
//! function may not expect. For instance, `parseInt` interprets the second
//! argument as a radix, causing `["1","2","3"].map(parseInt)` to produce
//! `[1, NaN, NaN]`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Array methods whose callbacks receive extra positional arguments.
const ITERATION_METHODS: &[&str] = &[
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
    "sort",
    "reduce",
    "reduceRight",
];

/// Flags function references passed directly to array iteration methods.
#[derive(Debug)]
pub struct NoArrayCallbackReference;

impl LintRule for NoArrayCallbackReference {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-callback-reference".to_owned(),
            description: "Disallow passing function references directly to array iteration methods"
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
        if !ITERATION_METHODS.contains(&method_name) {
            return;
        }

        // Must have at least one argument
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        // Flag only if the first argument is a bare identifier reference
        // (not an arrow function, function expression, or other expression)
        if let Some(AstNode::IdentifierReference(id)) = ctx.node(*first_arg_id) {
            let fn_name = id.name.as_str();
            let method_name_owned = method_name.to_owned();
            let call_span = Span::new(call.span.start, call.span.end);
            let fn_name_owned = fn_name.to_owned();
            ctx.report(Diagnostic {
                rule_name: "no-array-callback-reference".to_owned(),
                message: format!(
                    "Do not pass `{fn_name_owned}` directly to `.{method_name_owned}()` — it may receive unexpected arguments"
                ),
                span: call_span,
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

    starlint_rule_framework::lint_rule_test!(NoArrayCallbackReference);

    #[test]
    fn test_flags_map_parse_int() {
        let diags = lint("arr.map(parseInt);");
        assert_eq!(diags.len(), 1, "arr.map(parseInt) should be flagged");
    }

    #[test]
    fn test_flags_filter_boolean() {
        let diags = lint("arr.filter(Boolean);");
        assert_eq!(diags.len(), 1, "arr.filter(Boolean) should be flagged");
    }

    #[test]
    fn test_flags_some_with_identifier() {
        let diags = lint("arr.some(isValid);");
        assert_eq!(diags.len(), 1, "arr.some(isValid) should be flagged");
    }

    #[test]
    fn test_flags_reduce_with_identifier() {
        let diags = lint("arr.reduce(merge);");
        assert_eq!(diags.len(), 1, "arr.reduce(merge) should be flagged");
    }

    #[test]
    fn test_allows_arrow_function() {
        let diags = lint("arr.map(x => parseInt(x));");
        assert!(
            diags.is_empty(),
            "arrow function callback should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_expression() {
        let diags = lint("arr.map(function(x) { return x; });");
        assert!(
            diags.is_empty(),
            "function expression callback should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("arr.indexOf(parseInt);");
        assert!(
            diags.is_empty(),
            "non-iteration method should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_arguments() {
        let diags = lint("arr.sort();");
        assert!(
            diags.is_empty(),
            "iteration method without arguments should not be flagged"
        );
    }
}
