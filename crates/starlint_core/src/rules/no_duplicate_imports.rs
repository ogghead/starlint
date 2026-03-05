//! Rule: `no-duplicate-imports`
//!
//! Disallow duplicate module imports. If a module is imported more than
//! once, the imports should be merged into a single import statement.

use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

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

        let mut seen = HashSet::new();

        for stmt in &program.body {
            let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt else {
                continue;
            };
            let source = import.source.value.as_str();
            if !seen.insert(source.to_owned()) {
                ctx.report(Diagnostic {
                    rule_name: "no-duplicate-imports".to_owned(),
                    message: format!("'{source}' import is duplicated"),
                    span: Span::new(import.span.start, import.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
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
