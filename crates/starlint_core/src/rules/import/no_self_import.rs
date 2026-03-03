//! Rule: `import/no-self-import`
//!
//! Forbid a module from importing itself. Self-imports are always a mistake
//! and can cause runtime errors or infinite loops.
//!
//! This is a limited implementation — it checks if the import source
//! matches the file's own name without full path resolution.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags imports that appear to reference the current file.
#[derive(Debug)]
pub struct NoSelfImport;

impl NativeRule for NoSelfImport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-self-import".to_owned(),
            description: "Forbid a module from importing itself".to_owned(),
            category: Category::Correctness,
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

        let source_value = import.source.value.as_str();

        // Only check relative imports
        if !source_value.starts_with("./") && !source_value.starts_with("../") {
            return;
        }

        // Extract the file stem from the current file path
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Extract the imported file stem from the source
        // e.g., `./foo` -> `foo`, `./bar/baz` -> `baz`
        let import_stem = source_value
            .rsplit('/')
            .next()
            .unwrap_or("")
            .split('.')
            .next()
            .unwrap_or("");

        // Heuristic: if the imported stem matches the file stem and
        // the import is a same-directory relative import
        if source_value.starts_with("./")
            && !source_value.contains("/..")
            && source_value.matches('/').count() == 1
            && import_stem == file_stem
        {
            ctx.report_warning(
                "import/no-self-import",
                "Module should not import itself",
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSelfImport)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_self_import() {
        let diags = lint_with_path(r#"import foo from "./test";"#, Path::new("test.ts"));
        assert_eq!(diags.len(), 1, "self-import should be flagged");
    }

    #[test]
    fn test_allows_different_module() {
        let diags = lint_with_path(r#"import foo from "./other";"#, Path::new("test.ts"));
        assert!(
            diags.is_empty(),
            "import of different module should not be flagged"
        );
    }

    #[test]
    fn test_allows_package_import() {
        let diags = lint_with_path(r#"import foo from "lodash";"#, Path::new("test.ts"));
        assert!(diags.is_empty(), "package import should not be flagged");
    }
}
