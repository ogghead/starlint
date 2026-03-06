//! Rule: `typescript/ban-types`
//!
//! Disallow certain built-in types that are problematic as type annotations.
//! The uppercase wrapper types `Object`, `Boolean`, `Number`, `String`,
//! `Symbol`, `BigInt`, and `Function` should not be used — prefer their
//! lowercase primitive equivalents (`object`, `boolean`, `number`, `string`,
//! `symbol`, `bigint`) or more specific function signatures.

use oxc_ast::AstKind;
use oxc_ast::ast::TSTypeName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for BanTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/ban-types".to_owned(),
            description: "Disallow certain built-in types that are problematic".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSTypeReference])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSTypeReference(type_ref) = kind else {
            return;
        };

        let TSTypeName::IdentifierReference(ident) = &type_ref.type_name else {
            return;
        };

        let name = ident.name.as_str();

        for &(banned, message, replacement) in BANNED_TYPES {
            if name == banned {
                let ident_span = Span::new(ident.span.start, ident.span.end);
                ctx.report(Diagnostic {
                    rule_name: "typescript/ban-types".to_owned(),
                    message: message.to_owned(),
                    span: Span::new(type_ref.span.start, type_ref.span.end),
                    severity: Severity::Warning,
                    help: Some(message.to_owned()),
                    fix: replacement.map(|r| Fix {
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BanTypes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
