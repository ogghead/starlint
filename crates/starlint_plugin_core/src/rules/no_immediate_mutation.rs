//! Rule: `no-immediate-mutation`
//!
//! Disallows immediately mutating the result of a method that returns a new
//! array. For example, `arr.filter(x => x > 1).sort()` calls `.sort()` on
//! the new array returned by `.filter()`, which mutates it in place and
//! discards readability. Prefer `toSorted()` or assign to a variable first.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Methods that mutate an array in place.
const MUTATING_METHODS: &[&str] = &[
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "sort",
    "reverse",
    "fill",
    "copyWithin",
];

/// Methods that return a new array (the result is safe to use but mutating
/// it immediately is suspicious).
const NEW_ARRAY_METHODS: &[&str] = &[
    "filter",
    "map",
    "slice",
    "concat",
    "flat",
    "flatMap",
    "toSorted",
    "toReversed",
    "toSpliced",
    "with",
];

/// Flags chained calls like `arr.filter(...).sort()` where a mutating method
/// is called immediately on a freshly-created array.
#[derive(Debug)]
pub struct NoImmediateMutation;

impl LintRule for NoImmediateMutation {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-immediate-mutation".to_owned(),
            description:
                "Disallow immediately mutating the result of a method that returns a new array"
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

        // Outer call must be `<expr>.<mutatingMethod>(...)`
        let Some(AstNode::StaticMemberExpression(outer_member)) = ctx.node(call.callee) else {
            return;
        };

        let mutating_method = outer_member.property.as_str();
        if !MUTATING_METHODS.contains(&mutating_method) {
            return;
        }

        // The object of the outer member must be a call expression:
        // `<expr>.<newArrayMethod>(...)`
        let Some(AstNode::CallExpression(inner_call)) = ctx.node(outer_member.object) else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(inner_member)) = ctx.node(inner_call.callee)
        else {
            return;
        };

        let inner_method = inner_member.property.as_str();
        if !NEW_ARRAY_METHODS.contains(&inner_method) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-immediate-mutation".to_owned(),
            message: format!(
                "Immediately calling `.{mutating_method}()` on the result of `.{inner_method}()` mutates the new array in place"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoImmediateMutation);

    #[test]
    fn test_flags_filter_sort() {
        let diags = lint("[1,2,3].filter(x => x > 1).sort();");
        assert_eq!(diags.len(), 1, ".filter().sort() should be flagged");
    }

    #[test]
    fn test_flags_slice_reverse() {
        let diags = lint("arr.slice().reverse();");
        assert_eq!(diags.len(), 1, ".slice().reverse() should be flagged");
    }

    #[test]
    fn test_flags_map_push() {
        let diags = lint("arr.map(x => x).push(1);");
        assert_eq!(diags.len(), 1, ".map().push() should be flagged");
    }

    #[test]
    fn test_flags_concat_fill() {
        let diags = lint("arr.concat([1]).fill(0);");
        assert_eq!(diags.len(), 1, ".concat().fill() should be flagged");
    }

    #[test]
    fn test_allows_sort_alone() {
        let diags = lint("arr.sort();");
        assert!(diags.is_empty(), "standalone .sort() should not be flagged");
    }

    #[test]
    fn test_allows_push_alone() {
        let diags = lint("arr.push(1);");
        assert!(diags.is_empty(), "standalone .push() should not be flagged");
    }

    #[test]
    fn test_allows_filter_alone() {
        let diags = lint("arr.filter(x => x > 1);");
        assert!(
            diags.is_empty(),
            "standalone .filter() should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_mutating_chain() {
        let diags = lint("arr.filter(x => x > 1).map(x => x * 2);");
        assert!(
            diags.is_empty(),
            "chaining non-mutating methods should not be flagged"
        );
    }
}
