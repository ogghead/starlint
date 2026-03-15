//! Rule: `no-useless-computed-key`
//!
//! Disallow unnecessary computed property keys in objects and classes.
//! `{["foo"]: 1}` is equivalent to `{foo: 1}` and the computed form
//! is unnecessarily complex.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags computed property keys that use a literal value unnecessarily.
#[derive(Debug)]
pub struct NoUselessComputedKey;

impl LintRule for NoUselessComputedKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-computed-key".to_owned(),
            description: "Disallow unnecessary computed property keys".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::MethodDefinition,
            AstNodeType::ObjectProperty,
            AstNodeType::PropertyDefinition,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (computed, key_id, prop_span) = match node {
            AstNode::ObjectProperty(prop) => (prop.computed, prop.key, prop.span),
            AstNode::MethodDefinition(method) => (method.computed, method.key, method.span),
            AstNode::PropertyDefinition(prop) => (prop.computed, prop.key, prop.span),
            _ => return,
        };

        let Some(key_node) = ctx.node(key_id) else {
            return;
        };

        if !computed || !is_literal_key(key_node) {
            return;
        }

        let source = ctx.source_text();
        let key_span = key_node.span();
        let key_start = usize::try_from(key_span.start).unwrap_or(0);
        let key_end = usize::try_from(key_span.end).unwrap_or(0);
        let key_source = source.get(key_start..key_end).unwrap_or("");

        // Find [ before the key and ] after it in the source.
        let before = source.get(..key_start).unwrap_or("");
        let after = source.get(key_end..).unwrap_or("");
        let open = before.rfind('[').map(|p| u32::try_from(p).unwrap_or(0));
        let close = after
            .find(']')
            .map(|p| u32::try_from(key_end.saturating_add(p).saturating_add(1)).unwrap_or(0));

        let fix = if let (Some(open_pos), Some(close_pos)) = (open, close) {
            Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove computed brackets".to_owned(),
                edits: vec![Edit {
                    span: Span::new(open_pos, close_pos),
                    replacement: key_source.to_owned(),
                }],
                is_snippet: false,
            })
        } else {
            None
        };

        ctx.report(Diagnostic {
            rule_name: "no-useless-computed-key".to_owned(),
            message: "Unnecessary computed property key — use a literal key instead".to_owned(),
            span: Span::new(prop_span.start, prop_span.end),
            severity: Severity::Warning,
            help: Some("Remove the computed brackets".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

/// Check if a property key node is a literal string or number.
const fn is_literal_key(key: &AstNode) -> bool {
    matches!(key, AstNode::StringLiteral(_) | AstNode::NumericLiteral(_))
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUselessComputedKey);

    #[test]
    fn test_flags_string_computed_key() {
        let diags = lint("var obj = { [\"foo\"]: 1 };");
        assert_eq!(diags.len(), 1, "computed string key should be flagged");
    }

    #[test]
    fn test_allows_variable_computed_key() {
        let diags = lint("var obj = { [foo]: 1 };");
        assert!(
            diags.is_empty(),
            "computed variable key should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_key() {
        let diags = lint("var obj = { foo: 1 };");
        assert!(diags.is_empty(), "regular key should not be flagged");
    }

    #[test]
    fn test_flags_number_computed_key() {
        let diags = lint("var obj = { [0]: 1 };");
        assert_eq!(diags.len(), 1, "computed number key should be flagged");
    }
}
