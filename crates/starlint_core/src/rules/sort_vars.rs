//! Rule: `sort-vars`
//!
//! Require variables within the same declaration to be sorted alphabetically.
//! For example, `var a, b, c;` is valid, but `var b, a, c;` is not.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations where declarators are not alphabetically sorted.
#[derive(Debug)]
pub struct SortVars;

impl NativeRule for SortVars {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "sort-vars".to_owned(),
            description: "Require variables within the same declaration to be sorted".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::VariableDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclaration(decl) = kind else {
            return;
        };

        // Only check multi-declarator statements: `var a, b, c;`
        if decl.declarations.len() < 2 {
            return;
        }

        // Only check var/let/const with simple identifier bindings
        // Skip destructuring patterns — ordering doesn't apply there
        let names: Vec<(&str, oxc_span::Span)> = decl
            .declarations
            .iter()
            .filter_map(|d| {
                d.id.get_binding_identifiers()
                    .first()
                    .map(|ident| (ident.name.as_str(), d.id.span()))
            })
            .collect();

        if names.len() < 2 {
            return;
        }

        // Check pairwise ordering (case-insensitive by default, matching ESLint)
        for pair in names.windows(2) {
            let Some(&(prev_name, _)) = pair.first() else {
                continue;
            };
            let Some(&(curr_name, curr_span)) = pair.get(1) else {
                continue;
            };

            if prev_name.to_lowercase() > curr_name.to_lowercase() {
                ctx.report_warning(
                    "sort-vars",
                    &format!(
                        "Variables within the same declaration should be sorted alphabetically. \
                         Expected '{curr_name}' to come before '{prev_name}'"
                    ),
                    Span::new(curr_span.start, curr_span.end),
                );
                // Report only the first violation per declaration
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SortVars)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_sorted_vars() {
        let diags = lint("var a, b, c;");
        assert!(diags.is_empty(), "sorted vars should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_vars() {
        let diags = lint("var b, a;");
        assert_eq!(diags.len(), 1, "unsorted vars should be flagged");
    }

    #[test]
    fn test_allows_single_var() {
        let diags = lint("var a;");
        assert!(diags.is_empty(), "single var should not be flagged");
    }

    #[test]
    fn test_allows_sorted_let() {
        let diags = lint("let alpha, beta, gamma;");
        assert!(diags.is_empty(), "sorted let should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_const() {
        let diags = lint("const z = 1, a = 2;");
        assert_eq!(diags.len(), 1, "unsorted const should be flagged");
    }

    #[test]
    fn test_case_insensitive() {
        let diags = lint("var a, B, c;");
        assert!(diags.is_empty(), "case-insensitive sorting should pass");
    }

    #[test]
    fn test_flags_case_insensitive_unsorted() {
        let diags = lint("var B, a;");
        assert_eq!(
            diags.len(),
            1,
            "case-insensitive unsorted should be flagged"
        );
    }

    #[test]
    fn test_separate_declarations_independent() {
        let diags = lint("var b; var a;");
        assert!(
            diags.is_empty(),
            "separate declarations should not affect each other"
        );
    }
}
