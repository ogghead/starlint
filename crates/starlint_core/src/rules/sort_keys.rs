//! Rule: `sort-keys`
//!
//! Require object keys to be sorted alphabetically within each object literal.
//! This promotes consistency and makes it easier to find keys in large objects.

use oxc_ast::AstKind;
use oxc_ast::ast::PropertyKey;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags object literals whose keys are not alphabetically sorted.
#[derive(Debug)]
pub struct SortKeys;

/// Extract a comparable string from a property key.
fn key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.to_string()),
        PropertyKey::StringLiteral(s) => Some(s.value.to_string()),
        PropertyKey::NumericLiteral(n) => Some(n.raw_str().to_string()),
        // Computed keys can't be statically sorted
        _ => None,
    }
}

impl NativeRule for SortKeys {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "sort-keys".to_owned(),
            description: "Require object keys to be sorted".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
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

        if obj.properties.len() < 2 {
            return;
        }

        // Extract static key names from properties (skip spread elements)
        let keys: Vec<(String, oxc_span::Span)> = obj
            .properties
            .iter()
            .filter_map(|prop| match prop {
                oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) => {
                    key_name(&p.key).map(|name| (name, p.key.span()))
                }
                // SpreadProperty doesn't have a sortable key
                oxc_ast::ast::ObjectPropertyKind::SpreadProperty(_) => None,
            })
            .collect();

        if keys.len() < 2 {
            return;
        }

        // Check pairwise ordering (case-insensitive)
        for pair in keys.windows(2) {
            let Some((prev_name, _)) = pair.first() else {
                continue;
            };
            let Some((curr_name, curr_span)) = pair.get(1) else {
                continue;
            };

            if prev_name.to_lowercase() > curr_name.to_lowercase() {
                ctx.report_warning(
                    "sort-keys",
                    &format!(
                        "Object keys should be sorted alphabetically. \
                         Expected '{curr_name}' to come before '{prev_name}'"
                    ),
                    Span::new(curr_span.start, curr_span.end),
                );
                // Report only the first violation per object
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SortKeys)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_sorted_keys() {
        let diags = lint("var obj = { a: 1, b: 2, c: 3 };");
        assert!(diags.is_empty(), "sorted keys should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_keys() {
        let diags = lint("var obj = { b: 1, a: 2 };");
        assert_eq!(diags.len(), 1, "unsorted keys should be flagged");
    }

    #[test]
    fn test_allows_single_key() {
        let diags = lint("var obj = { a: 1 };");
        assert!(diags.is_empty(), "single key should not be flagged");
    }

    #[test]
    fn test_allows_empty_object() {
        let diags = lint("var obj = {};");
        assert!(diags.is_empty(), "empty object should not be flagged");
    }

    #[test]
    fn test_case_insensitive() {
        let diags = lint("var obj = { a: 1, B: 2, c: 3 };");
        assert!(diags.is_empty(), "case-insensitive sorting should pass");
    }

    #[test]
    fn test_string_keys() {
        let diags = lint("var obj = { 'alpha': 1, 'beta': 2 };");
        assert!(diags.is_empty(), "sorted string keys should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_string_keys() {
        let diags = lint("var obj = { 'beta': 1, 'alpha': 2 };");
        assert_eq!(diags.len(), 1, "unsorted string keys should be flagged");
    }

    #[test]
    fn test_skips_computed_keys() {
        let diags = lint("var obj = { [b]: 1, a: 2 };");
        assert!(diags.is_empty(), "computed keys should be skipped");
    }

    #[test]
    fn test_nested_objects_independent() {
        let diags = lint("var obj = { a: { z: 1, y: 2 }, b: 1 };");
        // Outer: a, b — sorted. Inner: z, y — unsorted.
        assert_eq!(
            diags.len(),
            1,
            "nested object should be checked independently"
        );
    }
}
