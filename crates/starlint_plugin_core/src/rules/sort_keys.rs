//! Rule: `sort-keys`
//!
//! Require object keys to be sorted alphabetically within each object literal.
//! This promotes consistency and makes it easier to find keys in large objects.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags object literals whose keys are not alphabetically sorted.
#[derive(Debug)]
pub struct SortKeys;

/// Extract a comparable string from a property key node.
fn key_name(key_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let node = ctx.node(key_id)?;
    match node {
        AstNode::IdentifierReference(id) => Some(id.name.clone()),
        AstNode::BindingIdentifier(id) => Some(id.name.clone()),
        AstNode::StringLiteral(s) => Some(s.value.clone()),
        AstNode::NumericLiteral(n) => {
            let start = usize::try_from(n.span.start).unwrap_or(0);
            let end = usize::try_from(n.span.end).unwrap_or(0);
            ctx.source_text().get(start..end).map(String::from)
        }
        // Computed keys can't be statically sorted
        _ => None,
    }
}

impl LintRule for SortKeys {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "sort-keys".to_owned(),
            description: "Require object keys to be sorted".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ObjectExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ObjectExpression(obj) = node else {
            return;
        };

        if obj.properties.len() < 2 {
            return;
        }

        // Extract static key names from properties (skip spread elements)
        let keys: Vec<(String, starlint_ast::types::Span)> = obj
            .properties
            .iter()
            .filter_map(|prop_id| {
                if let Some(AstNode::ObjectProperty(p)) = ctx.node(*prop_id) {
                    // Skip computed properties — they can't be statically sorted
                    if p.computed {
                        return None;
                    }
                    let key_span = ctx.node(p.key).map_or(
                        starlint_ast::types::Span::EMPTY,
                        starlint_ast::AstNode::span,
                    );
                    key_name(p.key, ctx).map(|name| (name, key_span))
                } else {
                    // SpreadProperty doesn't have a sortable key
                    None
                }
            })
            .collect();

        if keys.len() < 2 {
            return;
        }

        // Check pairwise ordering (case-insensitive)
        for pair in keys.windows(2) {
            let Some((prev_name, _)) = pair.first() else {
                continue;
            };
            let Some((curr_name, curr_span)) = pair.get(1) else {
                continue;
            };

            if prev_name.to_lowercase() > curr_name.to_lowercase() {
                ctx.report(Diagnostic {
                    rule_name: "sort-keys".to_owned(),
                    message: format!(
                        "Object keys should be sorted alphabetically. \
                         Expected '{curr_name}' to come before '{prev_name}'"
                    ),
                    span: Span::new(curr_span.start, curr_span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                // Report only the first violation per object
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(SortKeys);

    #[test]
    fn test_allows_sorted_keys() {
        let diags = lint("var obj = { a: 1, b: 2, c: 3 };");
        assert!(diags.is_empty(), "sorted keys should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_keys() {
        let diags = lint("var obj = { b: 1, a: 2 };");
        assert_eq!(diags.len(), 1, "unsorted keys should be flagged");
    }

    #[test]
    fn test_allows_single_key() {
        let diags = lint("var obj = { a: 1 };");
        assert!(diags.is_empty(), "single key should not be flagged");
    }

    #[test]
    fn test_allows_empty_object() {
        let diags = lint("var obj = {};");
        assert!(diags.is_empty(), "empty object should not be flagged");
    }

    #[test]
    fn test_case_insensitive() {
        let diags = lint("var obj = { a: 1, B: 2, c: 3 };");
        assert!(diags.is_empty(), "case-insensitive sorting should pass");
    }

    #[test]
    fn test_string_keys() {
        let diags = lint("var obj = { 'alpha': 1, 'beta': 2 };");
        assert!(diags.is_empty(), "sorted string keys should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_string_keys() {
        let diags = lint("var obj = { 'beta': 1, 'alpha': 2 };");
        assert_eq!(diags.len(), 1, "unsorted string keys should be flagged");
    }

    #[test]
    fn test_skips_computed_keys() {
        let diags = lint("var obj = { [b]: 1, a: 2 };");
        assert!(diags.is_empty(), "computed keys should be skipped");
    }

    #[test]
    fn test_nested_objects_independent() {
        let diags = lint("var obj = { a: { z: 1, y: 2 }, b: 1 };");
        // Outer: a, b — sorted. Inner: z, y — unsorted.
        assert_eq!(
            diags.len(),
            1,
            "nested object should be checked independently"
        );
    }
}
