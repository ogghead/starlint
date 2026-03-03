//! Rule: `require-module-attributes`
//!
//! Flag import declarations for JSON, CSS, or WASM modules that are missing
//! import attributes (also known as import assertions). Non-JS modules
//! should use `with { type: '...' }` to declare their type.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// File extensions that require import attributes.
const EXTENSIONS_NEEDING_ATTRIBUTES: &[&str] = &[".json", ".css", ".wasm"];

/// Flags non-JS module imports that are missing import attributes.
#[derive(Debug)]
pub struct RequireModuleAttributes;

impl NativeRule for RequireModuleAttributes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-module-attributes".to_owned(),
            description: "Require import attributes for non-JS modules".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source = import.source.value.as_str();

        // Check if the import source ends with a non-JS extension
        let needs_attributes = EXTENSIONS_NEEDING_ATTRIBUTES
            .iter()
            .any(|ext| source.ends_with(ext));

        if !needs_attributes {
            return;
        }

        // Check if import has a `with` clause with at least one attribute
        let has_attributes = import
            .with_clause
            .as_ref()
            .is_some_and(|clause| !clause.with_entries.is_empty());

        if !has_attributes {
            ctx.report_warning(
                "require-module-attributes",
                &format!("Import from '{source}' is missing import attributes"),
                Span::new(import.span.start, import.span.end),
            );
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

    /// Helper to lint source code as an ES module.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.mjs")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireModuleAttributes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.mjs"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_json_import_without_attributes() {
        let diags = lint("import data from './data.json';");
        assert_eq!(
            diags.len(),
            1,
            "JSON import without attributes should be flagged"
        );
    }

    #[test]
    fn test_allows_json_import_with_attributes() {
        let diags = lint("import data from './data.json' with { type: 'json' };");
        assert!(
            diags.is_empty(),
            "JSON import with attributes should not be flagged"
        );
    }

    #[test]
    fn test_allows_js_import_without_attributes() {
        let diags = lint("import foo from './foo.js';");
        assert!(
            diags.is_empty(),
            "JS import without attributes should not be flagged"
        );
    }

    #[test]
    fn test_flags_css_import_without_attributes() {
        let diags = lint("import styles from './styles.css';");
        assert_eq!(
            diags.len(),
            1,
            "CSS import without attributes should be flagged"
        );
    }

    #[test]
    fn test_flags_wasm_import_without_attributes() {
        let diags = lint("import mod from './module.wasm';");
        assert_eq!(
            diags.len(),
            1,
            "WASM import without attributes should be flagged"
        );
    }

    #[test]
    fn test_allows_bare_module_import() {
        let diags = lint("import foo from 'lodash';");
        assert!(diags.is_empty(), "bare module import should not be flagged");
    }
}
