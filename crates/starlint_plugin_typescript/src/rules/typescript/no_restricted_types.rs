//! Rule: `typescript/no-restricted-types`
//!
//! Disallow specific types from being used. Certain types like `Object` and
//! `{}` are almost never what the developer intends and should be replaced
//! with more specific alternatives such as `object`, `Record<string, unknown>`,
//! or a concrete type.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Default banned type names: (`banned_name`, message, replacement).
const BANNED_TYPE_NAMES: &[(&str, &str, &str)] = &[
    (
        "Object",
        "The `Object` type is too broad — use `object` or `Record<string, unknown>` instead",
        "object",
    ),
    (
        "Boolean",
        "Use lowercase `boolean` instead of the `Boolean` wrapper type",
        "boolean",
    ),
    (
        "Number",
        "Use lowercase `number` instead of the `Number` wrapper type",
        "number",
    ),
    (
        "String",
        "Use lowercase `string` instead of the `String` wrapper type",
        "string",
    ),
    (
        "Symbol",
        "Use lowercase `symbol` instead of the `Symbol` wrapper type",
        "symbol",
    ),
];

/// Flags usage of restricted type names and empty object type literals.
#[derive(Debug)]
pub struct NoRestrictedTypes;

impl LintRule for NoRestrictedTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-restricted-types".to_owned(),
            description: "Disallow specific types from being used".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeLiteral, AstNodeType::TSTypeReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::TSTypeReference(reference) => {
                check_type_reference(reference, ctx);
            }
            AstNode::TSTypeLiteral(lit) => {
                check_empty_object_type(lit, ctx);
            }
            _ => {}
        }
    }
}

/// Check if a type reference uses a banned type name.
fn check_type_reference(
    reference: &starlint_ast::node::TSTypeReferenceNode,
    ctx: &mut LintContext<'_>,
) {
    // TSTypeReferenceNode has type_name: String
    let name = reference.type_name.as_str();

    for &(banned, message, replacement) in BANNED_TYPE_NAMES {
        if name == banned {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-restricted-types".to_owned(),
                message: message.to_owned(),
                span: Span::new(reference.span.start, reference.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace `{banned}` with `{replacement}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(reference.span.start, reference.span.end),
                        replacement: replacement.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
            return;
        }
    }
}

/// Check if a type literal is an empty `{}` which is equivalent to any
/// non-nullish value.
fn check_empty_object_type(lit: &starlint_ast::node::TSTypeLiteralNode, ctx: &mut LintContext<'_>) {
    if !lit.members.is_empty() {
        return;
    }

    // Only flag truly empty `{}` — not index signature types
    ctx.report(Diagnostic {
        rule_name: "typescript/no-restricted-types".to_owned(),
        message:
            "The `{}` type means any non-nullish value — use `object` or `Record<string, unknown>` instead"
                .to_owned(),
        span: Span::new(lit.span.start, lit.span.end),
        severity: Severity::Warning,
        help: Some("Replace `{}` with `object` or `Record<string, unknown>`".to_owned()),
        fix: Some(Fix {
            kind: FixKind::SafeFix,
            message: "Replace with `object`".to_owned(),
            edits: vec![Edit {
                span: Span::new(lit.span.start, lit.span.end),
                replacement: "object".to_owned(),
            }],
            is_snippet: false,
        }),
        labels: vec![],
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoRestrictedTypes, "test.ts");

    #[test]
    fn test_flags_object_type() {
        let diags = lint("let x: Object;");
        assert_eq!(diags.len(), 1, "uppercase `Object` type should be flagged");
    }

    #[test]
    fn test_flags_string_wrapper_type() {
        let diags = lint("let x: String;");
        assert_eq!(
            diags.len(),
            1,
            "uppercase `String` wrapper type should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_object_type() {
        let diags = lint("let x: {} = y;");
        assert_eq!(diags.len(), 1, "empty object type should be flagged");
    }

    #[test]
    fn test_allows_lowercase_object() {
        let diags = lint("let x: object;");
        assert!(diags.is_empty(), "lowercase `object` should not be flagged");
    }

    #[test]
    fn test_allows_record_type() {
        let diags = lint("let x: Record<string, unknown>;");
        assert!(diags.is_empty(), "`Record` type should not be flagged");
    }
}
