//! Rule: `no-duplicate-imports`
//!
//! Disallow duplicate module imports. If a module is imported more than
//! once, the imports should be merged into a single import statement.

use std::collections::HashMap;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags duplicate import declarations from the same module.
#[derive(Debug)]
pub struct NoDuplicateImports;

impl NativeRule for NoDuplicateImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-duplicate-imports".to_owned(),
            description: "Disallow duplicate module imports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Program])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Program(program) = kind else {
            return;
        };

        // Map module source → first import span
        let mut seen: HashMap<String, Span> = HashMap::new();

        // Collect duplicates first to avoid borrow conflict with ctx
        let mut duplicates: Vec<(String, Span, Span)> = Vec::new();

        for stmt in &program.body {
            let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt else {
                continue;
            };
            let source_str = import.source.value.as_str();
            let import_span = Span::new(import.span.start, import.span.end);

            if let Some(&first_span) = seen.get(source_str) {
                duplicates.push((source_str.to_owned(), first_span, import_span));
            } else {
                seen.insert(source_str.to_owned(), import_span);
            }
        }

        // Build fixes first (immutable borrow of source_text), then report
        let diagnostics: Vec<Diagnostic> = {
            let source_text = ctx.source_text();
            duplicates
                .iter()
                .map(|(module_source, first_span, dup_span)| {
                    let edits = fix_utils::merge_import_edits(source_text, *first_span, *dup_span);
                    let fix = FixBuilder::new("Merge into first import")
                        .edits(edits)
                        .build();
                    Diagnostic {
                        rule_name: "no-duplicate-imports".to_owned(),
                        message: format!("'{module_source}' import is duplicated"),
                        span: *dup_span,
                        severity: Severity::Warning,
                        help: None,
                        fix,
                        labels: vec![],
                    }
                })
                .collect()
        };
        for diag in diagnostics {
            ctx.report(diag);
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.mjs")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDuplicateImports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.mjs"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_import() {
        let diags = lint("import { a } from 'foo';\nimport { b } from 'foo';");
        assert_eq!(diags.len(), 1, "duplicate import should be flagged");
    }

    #[test]
    fn test_allows_different_sources() {
        let diags = lint("import { a } from 'foo';\nimport { b } from 'bar';");
        assert!(diags.is_empty(), "different sources should not be flagged");
    }

    #[test]
    fn test_allows_single_import() {
        let diags = lint("import { a, b } from 'foo';");
        assert!(diags.is_empty(), "single import should not be flagged");
    }

    #[test]
    fn test_flags_triple_import() {
        let diags =
            lint("import { a } from 'foo';\nimport { b } from 'foo';\nimport { c } from 'foo';");
        assert_eq!(
            diags.len(),
            2,
            "two duplicate imports should produce two diagnostics"
        );
    }
}
