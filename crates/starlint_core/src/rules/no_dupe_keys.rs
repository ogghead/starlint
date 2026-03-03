//! Rule: `no-dupe-keys`
//!
//! Disallow duplicate keys in object literals. Multiple properties with the
//! same key in an object literal cause the last one to silently overwrite
//! earlier ones, which is almost always a mistake.

use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_ast::ast::{PropertyKey, PropertyKind};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags object literals with duplicate property keys.
#[derive(Debug)]
pub struct NoDupeKeys;

impl NativeRule for NoDupeKeys {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-dupe-keys".to_owned(),
            description: "Disallow duplicate keys in object literals".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ObjectExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ObjectExpression(obj) = kind else {
            return;
        };

        let mut seen = HashSet::new();

        for property in &obj.properties {
            let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) = property else {
                // SpreadProperty — skip
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

            let Some(key_name) = static_property_key_name(&prop.key, ctx.source_text()) else {
                continue;
            };

            if !seen.insert(key_name.clone()) {
                let key_span = prop.key.span();
                ctx.report_error(
                    "no-dupe-keys",
                    &format!("Duplicate key `{key_name}`"),
                    Span::new(key_span.start, key_span.end),
                );
            }
        }
    }
}

/// Extract a static key name from a property key.
fn static_property_key_name(key: &PropertyKey<'_>, source: &str) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
        PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
        PropertyKey::NumericLiteral(lit) => {
            // Use source text to preserve the original representation
            let start = usize::try_from(lit.span.start).unwrap_or(0);
            let end = usize::try_from(lit.span.end).unwrap_or(0);
            source.get(start..end).map(String::from)
        }
        _ => None,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDupeKeys)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

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
