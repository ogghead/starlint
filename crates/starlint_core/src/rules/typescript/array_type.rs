//! Rule: `typescript/array-type`
//!
//! Enforce consistent array type style. By default, enforces the "array" style
//! where `Array<T>` should be written as `T[]` and `ReadonlyArray<T>` should be
//! written as `readonly T[]`. Generic wrapper types like `Array<T>` are more
//! verbose and less conventional in most `TypeScript` codebases.

use oxc_ast::AstKind;
use oxc_ast::ast::TSTypeName;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Type names that should use shorthand array syntax instead.
const ARRAY_TYPE_NAMES: &[&str] = &["Array", "ReadonlyArray"];

/// Flags `Array<T>` and `ReadonlyArray<T>` type references, preferring `T[]`
/// and `readonly T[]` shorthand syntax.
#[derive(Debug)]
pub struct ArrayType;

impl NativeRule for ArrayType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/array-type".to_owned(),
            description: "Enforce consistent array type style (`T[]` instead of `Array<T>`)"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSTypeReference(type_ref) = kind else {
            return;
        };

        let TSTypeName::IdentifierReference(ident) = &type_ref.type_name else {
            return;
        };

        let name = ident.name.as_str();

        if !ARRAY_TYPE_NAMES.contains(&name) {
            return;
        }

        let suggestion = if name == "ReadonlyArray" {
            "Use `readonly T[]` instead of `ReadonlyArray<T>`"
        } else {
            "Use `T[]` instead of `Array<T>`"
        };

        ctx.report_warning(
            "typescript/array-type",
            suggestion,
            Span::new(type_ref.span.start, type_ref.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ArrayType)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_array_generic() {
        let diags = lint("let x: Array<number>;");
        assert_eq!(diags.len(), 1, "`Array<number>` should be flagged");
    }

    #[test]
    fn test_flags_readonly_array_generic() {
        let diags = lint("let x: ReadonlyArray<string>;");
        assert_eq!(diags.len(), 1, "`ReadonlyArray<string>` should be flagged");
    }

    #[test]
    fn test_allows_shorthand_array() {
        let diags = lint("let x: number[];");
        assert!(
            diags.is_empty(),
            "`number[]` shorthand should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_generic_types() {
        let diags = lint("let x: Map<string, number>;");
        assert!(diags.is_empty(), "`Map<K, V>` should not be flagged");
    }

    #[test]
    fn test_flags_nested_array_generic() {
        let diags = lint("let x: Array<Array<number>>;");
        assert_eq!(
            diags.len(),
            2,
            "both nested `Array<>` references should be flagged"
        );
    }
}
