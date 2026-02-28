//! Rule: `typescript/no-wrapper-object-types`
//!
//! Disallow wrapper object types (`String`, `Number`, `Boolean`, `BigInt`,
//! `Symbol`) in type annotations. These uppercase types refer to the boxed
//! object wrappers, not the primitive types. Almost all TypeScript code should
//! use the lowercase primitive forms (`string`, `number`, `boolean`, `bigint`,
//! `symbol`) instead.

use oxc_ast::AstKind;
use oxc_ast::ast::TSTypeName;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-wrapper-object-types";

/// Wrapper object types mapped to their primitive equivalents.
const WRAPPER_TYPES: &[(&str, &str)] = &[
    ("String", "string"),
    ("Number", "number"),
    ("Boolean", "boolean"),
    ("BigInt", "bigint"),
    ("Symbol", "symbol"),
];

/// Flags `TSTypeReference` nodes that refer to wrapper object types instead
/// of their lowercase primitive equivalents.
#[derive(Debug)]
pub struct NoWrapperObjectTypes;

impl NativeRule for NoWrapperObjectTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow wrapper object types in type annotations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSTypeReference(type_ref) = kind else {
            return;
        };

        // Only check simple identifier references (not qualified names like `Foo.Bar`)
        let TSTypeName::IdentifierReference(ident) = &type_ref.type_name else {
            return;
        };

        let name = ident.name.as_str();

        for &(wrapper, primitive) in WRAPPER_TYPES {
            if name == wrapper {
                ctx.report_error(
                    RULE_NAME,
                    &format!("Use lowercase `{primitive}` instead of `{wrapper}`"),
                    Span::new(type_ref.span.start, type_ref.span.end),
                );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoWrapperObjectTypes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_wrapper() {
        let diags = lint("let x: String;");
        assert_eq!(diags.len(), 1, "`String` wrapper type should be flagged");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("string")),
            "message should suggest lowercase `string`"
        );
    }

    #[test]
    fn test_flags_number_wrapper() {
        let diags = lint("function f(n: Number): void {}");
        assert_eq!(diags.len(), 1, "`Number` wrapper type should be flagged");
    }

    #[test]
    fn test_flags_boolean_wrapper() {
        let diags = lint("const b: Boolean = true;");
        assert_eq!(diags.len(), 1, "`Boolean` wrapper type should be flagged");
    }

    #[test]
    fn test_flags_bigint_and_symbol_wrappers() {
        let diags = lint("type Pair = { a: BigInt; b: Symbol };");
        assert_eq!(
            diags.len(),
            2,
            "both `BigInt` and `Symbol` wrapper types should be flagged"
        );
    }

    #[test]
    fn test_allows_lowercase_primitives() {
        let diags = lint("let x: string; let y: number; let z: boolean;");
        assert!(
            diags.is_empty(),
            "lowercase primitive types should not be flagged"
        );
    }

    #[test]
    fn test_allows_custom_types() {
        let diags = lint("interface MyString {} let x: MyString;");
        assert!(
            diags.is_empty(),
            "custom types that are not wrapper types should not be flagged"
        );
    }
}
