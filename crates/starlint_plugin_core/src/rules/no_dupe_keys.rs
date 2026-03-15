//! Rule: `no-dupe-keys`
//!
//! Disallow duplicate keys in object literals. Multiple properties with the
//! same key in an object literal cause the last one to silently overwrite
//! earlier ones, which is almost always a mistake.

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::PropertyKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::ast_utils::extract_static_key_name;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags object literals with duplicate property keys.
#[derive(Debug)]
pub struct NoDupeKeys;

impl LintRule for NoDupeKeys {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-dupe-keys".to_owned(),
            description: "Disallow duplicate keys in object literals".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ObjectExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ObjectExpression(obj) = node else {
            return;
        };

        let mut seen = HashSet::new();

        for prop_id in &obj.properties {
            let Some(AstNode::ObjectProperty(prop)) = ctx.node(*prop_id) else {
                // SpreadElement — skip
                continue;
            };

            // Skip getters and setters — they use the same key intentionally
            if prop.kind != PropertyKind::Init {
                continue;
            }

            // Skip computed properties — we can't statically determine the key
            if prop.computed {
                continue;
            }

            let Some(key_name) = extract_static_key_name(prop.key, ctx) else {
                continue;
            };

            if !seen.insert(key_name.clone()) {
                let key_span = ctx.node(prop.key).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );
                ctx.report(Diagnostic {
                    rule_name: "no-dupe-keys".to_owned(),
                    message: format!("Duplicate key `{key_name}`"),
                    span: Span::new(key_span.start, key_span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoDupeKeys);

    #[test]
    fn test_flags_duplicate_key() {
        let diags = lint("const obj = { a: 1, a: 2 };");
        assert_eq!(diags.len(), 1, "duplicate key 'a' should be flagged");
    }

    #[test]
    fn test_flags_duplicate_string_key() {
        let diags = lint(r#"const obj = { "x": 1, "x": 2 };"#);
        assert_eq!(diags.len(), 1, "duplicate string key should be flagged");
    }

    #[test]
    fn test_flags_duplicate_number_key() {
        let diags = lint("const obj = { 1: 'a', 1: 'b' };");
        assert_eq!(diags.len(), 1, "duplicate number key should be flagged");
    }

    #[test]
    fn test_allows_unique_keys() {
        let diags = lint("const obj = { a: 1, b: 2, c: 3 };");
        assert!(diags.is_empty(), "unique keys should not be flagged");
    }

    #[test]
    fn test_allows_getter_setter_pair() {
        let diags = lint("const obj = { get x() {}, set x(v) {} };");
        assert!(diags.is_empty(), "getter/setter pair should not be flagged");
    }

    #[test]
    fn test_allows_computed_keys() {
        let diags = lint("const obj = { [a]: 1, [a]: 2 };");
        assert!(
            diags.is_empty(),
            "computed keys should not be flagged (can't determine statically)"
        );
    }

    #[test]
    fn test_allows_spread() {
        let diags = lint("const obj = { a: 1, ...other, a: 2 };");
        // The spread resets the object — but ESLint still flags this.
        // We flag it too since the static key 'a' appears twice.
        assert_eq!(
            diags.len(),
            1,
            "duplicate key across spread should be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_duplicates() {
        let diags = lint("const obj = { a: 1, b: 2, a: 3, b: 4 };");
        assert_eq!(
            diags.len(),
            2,
            "two pairs of duplicates should produce two diagnostics"
        );
    }
}
