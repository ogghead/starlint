//! Rule: `typescript/require-array-sort-compare`
//!
//! Require a compare function argument in `Array.prototype.sort()`. Without a
//! compare function, `sort()` converts elements to strings and sorts them
//! lexicographically, which is often not what you want for numeric or complex
//! data.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.sort()` calls with zero arguments.
#[derive(Debug)]
pub struct RequireArraySortCompare;

impl NativeRule for RequireArraySortCompare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/require-array-sort-compare".to_owned(),
            description: "Require a compare function argument in `Array.prototype.sort()`"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
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

        if call.arguments.is_empty() {
            ctx.report(Diagnostic {
                rule_name: "typescript/require-array-sort-compare".to_owned(),
                message: "Provide a compare function to `.sort()` — without one, elements are sorted as strings".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireArraySortCompare)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_sort_without_compare() {
        let diags = lint("const arr = [3, 1, 2]; arr.sort();");
        assert_eq!(
            diags.len(),
            1,
            "`.sort()` without a compare function should be flagged"
        );
    }

    #[test]
    fn test_allows_sort_with_compare() {
        let diags = lint("const arr = [3, 1, 2]; arr.sort((a, b) => a - b);");
        assert!(
            diags.is_empty(),
            "`.sort()` with a compare function should not be flagged"
        );
    }

    #[test]
    fn test_allows_sort_with_named_compare() {
        let diags = lint("const arr = [3, 1, 2]; arr.sort(compareFn);");
        assert!(
            diags.is_empty(),
            "`.sort()` with a named compare function should not be flagged"
        );
    }

    #[test]
    fn test_ignores_non_sort_method() {
        let diags = lint("arr.filter();");
        assert!(
            diags.is_empty(),
            "non-sort method calls should not be flagged"
        );
    }

    #[test]
    fn test_flags_chained_sort_without_compare() {
        let diags = lint("getItems().sort();");
        assert_eq!(
            diags.len(),
            1,
            "chained `.sort()` without compare should be flagged"
        );
    }
}
