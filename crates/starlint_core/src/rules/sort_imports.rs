//! Rule: `sort-imports`
//!
//! Require import declarations to be sorted alphabetically by their source
//! module specifier. Only checks the order of import declarations, not the
//! members within each import.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportDeclaration;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags import declarations that are not sorted by source specifier.
#[derive(Debug)]
pub struct SortImports;

impl NativeRule for SortImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "sort-imports".to_owned(),
            description: "Require import declarations to be sorted".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Program])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Program(program) = kind else {
            return;
        };

        // Collect all import declarations in order
        let imports: Vec<&ImportDeclaration<'_>> = program
            .body
            .iter()
            .filter_map(|stmt| {
                if let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt {
                    Some(import.as_ref())
                } else {
                    None
                }
            })
            .collect();

        if imports.len() < 2 {
            return;
        }

        // Check pairwise ordering by source specifier (case-insensitive)
        for pair in imports.windows(2) {
            let Some(prev) = pair.first() else {
                continue;
            };
            let Some(curr) = pair.get(1) else {
                continue;
            };

            let prev_source = prev.source.value.as_str();
            let curr_source = curr.source.value.as_str();

            if prev_source.to_lowercase() > curr_source.to_lowercase() {
                ctx.report(Diagnostic {
                    rule_name: "sort-imports".to_owned(),
                    message: format!(
                        "Import from '{curr_source}' should come before import from '{prev_source}'"
                    ),
                    span: Span::new(curr.span.start, curr.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                // Report only the first violation
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SortImports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_sorted_imports() {
        let diags = lint("import a from 'a';\nimport b from 'b';\nimport c from 'c';");
        assert!(diags.is_empty(), "sorted imports should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_imports() {
        let diags = lint("import b from 'b';\nimport a from 'a';");
        assert_eq!(diags.len(), 1, "unsorted imports should be flagged");
    }

    #[test]
    fn test_allows_single_import() {
        let diags = lint("import a from 'a';");
        assert!(diags.is_empty(), "single import should not be flagged");
    }

    #[test]
    fn test_case_insensitive_sort() {
        let diags = lint("import a from 'Alpha';\nimport b from 'beta';");
        assert!(diags.is_empty(), "case-insensitive sort should pass");
    }

    #[test]
    fn test_flags_case_insensitive_unsorted() {
        let diags = lint("import b from 'beta';\nimport a from 'Alpha';");
        assert_eq!(
            diags.len(),
            1,
            "case-insensitive unsorted should be flagged"
        );
    }

    #[test]
    fn test_allows_no_imports() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "no imports should not be flagged");
    }
}
