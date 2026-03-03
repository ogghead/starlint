//! Rule: `typescript/consistent-indexed-object-style`
//!
//! Enforce consistent usage of index signatures vs the `Record` utility type.
//! An index signature type like `{ [key: string]: T }` can be expressed more
//! concisely as `Record<string, T>`. This rule flags type literals that contain
//! exactly one index signature member and nothing else, suggesting `Record`
//! as the preferred alternative.

use oxc_ast::AstKind;
use oxc_ast::ast::TSSignature;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags type literals with a single index signature, preferring `Record<K, V>`.
#[derive(Debug)]
pub struct ConsistentIndexedObjectStyle;

impl NativeRule for ConsistentIndexedObjectStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/consistent-indexed-object-style".to_owned(),
            description: "Enforce `Record<K, V>` over index signature syntax".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSTypeLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSTypeLiteral(lit) = kind else {
            return;
        };

        // Only flag when there is exactly one member and it is an index
        // signature. Object types with additional properties or methods
        // are not equivalent to `Record`.
        if lit.members.len() != 1 {
            return;
        }

        let Some(member) = lit.members.first() else {
            return;
        };

        if !matches!(member, TSSignature::TSIndexSignature(_)) {
            return;
        }

        ctx.report_warning(
            "typescript/consistent-indexed-object-style",
            "Use `Record<K, V>` instead of an index signature — it is more concise and conventional",
            Span::new(lit.span.start, lit.span.end),
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentIndexedObjectStyle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

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
