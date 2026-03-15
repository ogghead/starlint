//! Rule: `typescript/consistent-indexed-object-style`
//!
//! Enforce consistent usage of index signatures vs the `Record` utility type.
//! An index signature type like `{ [key: string]: T }` can be expressed more
//! concisely as `Record<string, T>`. This rule flags type literals that contain
//! exactly one index signature member and nothing else, suggesting `Record`
//! as the preferred alternative.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags type literals with a single index signature, preferring `Record<K, V>`.
#[derive(Debug)]
pub struct ConsistentIndexedObjectStyle;

impl LintRule for ConsistentIndexedObjectStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/consistent-indexed-object-style".to_owned(),
            description: "Enforce `Record<K, V>` over index signature syntax".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeLiteral])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSTypeLiteral(lit) = node else {
            return;
        };

        // Only flag when there is exactly one member and it is an index
        // signature. Object types with additional properties or methods
        // are not equivalent to `Record`.
        if lit.members.len() != 1 {
            return;
        }

        // Use source text to detect index signature pattern `{ [key: K]: V }`
        // since the flat AST may not have a dedicated TSIndexSignature node.
        let source = ctx.source_text();
        let lit_text = &source[lit.span.start as usize..lit.span.end as usize];

        // An index signature contains `[` before `]:` — check for that pattern
        if !lit_text.contains('[') || !lit_text.contains("]:") {
            return;
        }

        // Try to extract K and V from `{ [key: K]: V }` to produce `Record<K, V>`
        let fix = extract_index_sig_types(lit_text).map(|(k, v)| {
            let replacement = format!("Record<{k}, {v}>");
            Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement,
                }],
                is_snippet: false,
            }
        });

        ctx.report(Diagnostic {
            rule_name: "typescript/consistent-indexed-object-style".to_owned(),
            message: "Use `Record<K, V>` instead of an index signature — it is more concise and conventional".to_owned(),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `Record<K, V>`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

/// Extract key type `K` and value type `V` from `{ [key: K]: V }` text.
fn extract_index_sig_types(text: &str) -> Option<(&str, &str)> {
    // Find `: K]: V` pattern inside the braces
    let colon_pos = text.find(": ")?;
    let bracket_close = text.find("]:")?;
    let key_type = text.get(colon_pos.saturating_add(2)..bracket_close)?.trim();
    let after_bracket_colon = text.get(bracket_close.saturating_add(2)..)?;
    // Value type is everything after `]: ` up to the closing `}`
    let value_part = after_bracket_colon.trim();
    let mut value_type = value_part.strip_suffix('}')?.trim();
    value_type = value_type.strip_suffix(';').unwrap_or(value_type).trim();
    if value_type.is_empty() {
        return None;
    }
    Some((key_type, value_type))
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(ConsistentIndexedObjectStyle, "test.ts");

    #[test]
    fn test_flags_index_signature_only() {
        let diags = lint("type Foo = { [key: string]: number };");
        assert_eq!(
            diags.len(),
            1,
            "type with only an index signature should be flagged"
        );
    }

    #[test]
    fn test_flags_inline_index_signature() {
        let diags = lint("let x: { [k: string]: boolean };");
        assert_eq!(
            diags.len(),
            1,
            "inline index signature type should be flagged"
        );
    }

    #[test]
    fn test_allows_record_type() {
        let diags = lint("type Foo = Record<string, number>;");
        assert!(diags.is_empty(), "`Record` type should not be flagged");
    }

    #[test]
    fn test_allows_index_with_other_members() {
        let diags = lint("type Foo = { [key: string]: number; length: number };");
        assert!(
            diags.is_empty(),
            "index signature with other members should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_object_type() {
        let diags = lint("type Foo = {};");
        assert!(
            diags.is_empty(),
            "empty object type should not be flagged by this rule"
        );
    }
}
