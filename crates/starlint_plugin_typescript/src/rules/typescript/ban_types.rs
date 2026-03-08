//! Rule: `typescript/ban-types`
//!
//! Disallow certain built-in types that are problematic as type annotations.
//! The uppercase wrapper types `Object`, `Boolean`, `Number`, `String`,
//! `Symbol`, `BigInt`, and `Function` should not be used — prefer their
//! lowercase primitive equivalents (`object`, `boolean`, `number`, `string`,
//! `symbol`, `bigint`) or more specific function signatures.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Banned type names, their suggested replacements, and optional auto-fix replacement.
/// The third element is `Some(replacement)` when a safe direct replacement exists.
const BANNED_TYPES: &[(&str, &str, Option<&str>)] = &[
    (
        "Object",
        "Use `object` or a more specific type instead of `Object`",
        Some("object"),
    ),
    (
        "Boolean",
        "Use `boolean` instead of `Boolean`",
        Some("boolean"),
    ),
    ("Number", "Use `number` instead of `Number`", Some("number")),
    ("String", "Use `string` instead of `String`", Some("string")),
    ("Symbol", "Use `symbol` instead of `Symbol`", Some("symbol")),
    ("BigInt", "Use `bigint` instead of `BigInt`", Some("bigint")),
    (
        "Function",
        "Use a specific function type like `() => void` instead of `Function`",
        None,
    ),
];

/// Flags usage of banned built-in types in type annotations.
#[derive(Debug)]
pub struct BanTypes;

impl LintRule for BanTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/ban-types".to_owned(),
            description: "Disallow certain built-in types that are problematic".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSTypeReference(type_ref) = node else {
            return;
        };

        let name = type_ref.type_name.as_str();

        for &(banned, message, replacement) in BANNED_TYPES {
            if name == banned {
                // Use the type reference span for the fix (replaces the whole type name)
                let ident_span = Span::new(type_ref.span.start, type_ref.span.end);
                ctx.report(Diagnostic {
                    rule_name: "typescript/ban-types".to_owned(),
                    message: message.to_owned(),
                    span: Span::new(type_ref.span.start, type_ref.span.end),
                    severity: Severity::Warning,
                    help: Some(message.to_owned()),
                    fix: replacement.map(|r| Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace `{banned}` with `{r}`"),
                        edits: vec![Edit {
                            span: ident_span,
                            replacement: r.to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(BanTypes)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_uppercase_string() {
        let diags = lint("let x: String;");
        assert_eq!(diags.len(), 1, "`String` type should be flagged");
    }

    #[test]
    fn test_flags_uppercase_number() {
        let diags = lint("let x: Number;");
        assert_eq!(diags.len(), 1, "`Number` type should be flagged");
    }

    #[test]
    fn test_flags_uppercase_boolean() {
        let diags = lint("let x: Boolean;");
        assert_eq!(diags.len(), 1, "`Boolean` type should be flagged");
    }

    #[test]
    fn test_flags_function_type() {
        let diags = lint("let f: Function;");
        assert_eq!(diags.len(), 1, "`Function` type should be flagged");
    }

    #[test]
    fn test_allows_lowercase_primitives() {
        let diags = lint("let a: string; let b: number; let c: boolean;");
        assert!(
            diags.is_empty(),
            "lowercase primitive types should not be flagged"
        );
    }
}
