//! Rule: `no-array-sort`
//!
//! Disallow `Array.prototype.sort()` which mutates the array in-place.
//! Prefer `toSorted()` which returns a new sorted array without modifying
//! the original.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.sort()` calls that mutate arrays in-place.
#[derive(Debug)]
pub struct NoArraySort;

impl NativeRule for NoArraySort {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-sort".to_owned(),
            description: "Disallow `.sort()` which mutates the array — prefer `.toSorted()`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "sort" {
            return;
        }

        ctx.report_warning(
            "no-array-sort",
            "`.sort()` mutates the array in-place — prefer `.toSorted()` instead",
            Span::new(call.span.start, call.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArraySort)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_sort_no_args() {
        let diags = lint("arr.sort();");
        assert_eq!(diags.len(), 1, ".sort() should be flagged");
    }

    #[test]
    fn test_flags_sort_with_comparator() {
        let diags = lint("arr.sort((a, b) => a - b);");
        assert_eq!(
            diags.len(),
            1,
            ".sort() with comparator should also be flagged (still mutates)"
        );
    }

    #[test]
    fn test_flags_spread_sort() {
        let diags = lint("[...arr].sort();");
        assert_eq!(
            diags.len(),
            1,
            "[...arr].sort() is still a .sort() call and should be flagged"
        );
    }

    #[test]
    fn test_allows_to_sorted() {
        let diags = lint("arr.toSorted();");
        assert!(diags.is_empty(), ".toSorted() should not be flagged");
    }

    #[test]
    fn test_allows_to_sorted_with_comparator() {
        let diags = lint("arr.toSorted((a, b) => a - b);");
        assert!(
            diags.is_empty(),
            ".toSorted() with comparator should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("arr.map(x => x);");
        assert!(
            diags.is_empty(),
            "unrelated method calls should not be flagged"
        );
    }
}
