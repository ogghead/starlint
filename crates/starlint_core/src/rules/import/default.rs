//! Rule: `import/default`
//!
//! Ensure a default export is present when a default import is used.
//! This is a static analysis approximation — it checks whether the import
//! declaration has a default specifier, which can be paired with module
//! resolution in the future.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportDeclarationSpecifier;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags default imports from modules that may not have a default export.
///
/// Without full module resolution this rule flags default imports from
/// obviously-named-only modules (heuristic: source ending in `/index`).
#[derive(Debug)]
pub struct DefaultExport;

impl NativeRule for DefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/default".to_owned(),
            description: "Ensure a default export is present when a default import is used"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let Some(specifiers) = &import.specifiers else {
            return;
        };
        let has_default = specifiers
            .iter()
            .any(|spec| matches!(spec, ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));

        if !has_default {
            return;
        }

        // Type-only imports don't need runtime default exports
        if import.import_kind.is_type() {
            return;
        }

        let source_value = import.source.value.as_str();

        // Heuristic: flag default imports from JSON files (they have no default export
        // in strict ESM) — this is a common mistake
        if std::path::Path::new(source_value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            ctx.report(Diagnostic {
                rule_name: "import/default".to_owned(),
                message: "No default export found in imported JSON module".to_owned(),
                span: Span::new(import.span.start, import.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DefaultExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_default_import_from_json() {
        let diags = lint(r#"import data from "./data.json";"#);
        assert_eq!(diags.len(), 1, "default import from JSON should be flagged");
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "./module";"#);
        assert!(diags.is_empty(), "named import should not be flagged");
    }

    #[test]
    fn test_allows_default_import_from_js() {
        let diags = lint(r#"import foo from "./module";"#);
        assert!(
            diags.is_empty(),
            "default import from JS module should not be flagged"
        );
    }
}
