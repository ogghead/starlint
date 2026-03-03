//! Rule: `typescript/no-restricted-types`
//!
//! Disallow specific types from being used. Certain types like `Object` and
//! `{}` are almost never what the developer intends and should be replaced
//! with more specific alternatives such as `object`, `Record<string, unknown>`,
//! or a concrete type.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default banned type names.
const BANNED_TYPE_NAMES: &[(&str, &str)] = &[
    (
        "Object",
        "The `Object` type is too broad — use `object` or `Record<string, unknown>` instead",
    ),
    (
        "Boolean",
        "Use lowercase `boolean` instead of the `Boolean` wrapper type",
    ),
    (
        "Number",
        "Use lowercase `number` instead of the `Number` wrapper type",
    ),
    (
        "String",
        "Use lowercase `string` instead of the `String` wrapper type",
    ),
    (
        "Symbol",
        "Use lowercase `symbol` instead of the `Symbol` wrapper type",
    ),
];

/// Flags usage of restricted type names and empty object type literals.
#[derive(Debug)]
pub struct NoRestrictedTypes;

impl NativeRule for NoRestrictedTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-restricted-types".to_owned(),
            description: "Disallow specific types from being used".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSTypeLiteral, AstType::TSTypeReference])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::TSTypeReference(reference) => {
                check_type_reference(reference, ctx);
            }
            AstKind::TSTypeLiteral(lit) => {
                check_empty_object_type(lit, ctx);
            }
            _ => {}
        }
    }
}

/// Check if a type reference uses a banned type name.
fn check_type_reference(
    reference: &oxc_ast::ast::TSTypeReference<'_>,
    ctx: &mut NativeLintContext<'_>,
) {
    let Some(ident) = reference.type_name.get_identifier_reference() else {
        return;
    };

    let name = ident.name.as_str();

    for &(banned, message) in BANNED_TYPE_NAMES {
        if name == banned {
            ctx.report_warning(
                "typescript/no-restricted-types",
                message,
                Span::new(reference.span.start, reference.span.end),
            );
            return;
        }
    }
}

/// Check if a type literal is an empty `{}` which is equivalent to any
/// non-nullish value.
fn check_empty_object_type(lit: &oxc_ast::ast::TSTypeLiteral<'_>, ctx: &mut NativeLintContext<'_>) {
    if !lit.members.is_empty() {
        return;
    }

    // Only flag truly empty `{}` — not index signature types
    ctx.report_warning(
        "typescript/no-restricted-types",
        "The `{}` type means any non-nullish value — use `object` or `Record<string, unknown>` instead",
        Span::new(lit.span.start, lit.span.end),
    );
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestrictedTypes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

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
