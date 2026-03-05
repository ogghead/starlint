//! Rule: `import/no-cycle`
//!
//! Detect circular import dependencies. Full cycle detection requires
//! module resolution across the entire dependency graph, which is not yet
//! available. As a useful stub, this rule flags self-imports — a module
//! importing from its own file path — which is the simplest form of cycle.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags self-imports as the simplest detectable import cycle.
#[derive(Debug)]
pub struct NoCycle;

impl NativeRule for NoCycle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-cycle".to_owned(),
            description: "Detect circular import dependencies (stub: flags self-imports)"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
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
        if !source_value.starts_with('.') {
            return;
        }

        // Extract the file stem of the current file
        let file_path = ctx.file_path();
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        // Extract the last segment of the import path (after the final /)
        let import_segment = source_value.rsplit('/').next().unwrap_or(source_value);

        // Strip extension from import segment if present
        let import_base = import_segment.rfind('.').map_or(import_segment, |pos| {
            import_segment.get(..pos).unwrap_or(import_segment)
        });

        // Self-import: the import source resolves to the same file
        // Match patterns like `./myModule`, `./myModule.ts`, `./myModule.js`
        let is_self_import = !file_stem.is_empty()
            && import_base == file_stem
            && (source_value == format!("./{file_stem}")
                || source_value == format!("./{file_stem}.ts")
                || source_value == format!("./{file_stem}.js")
                || source_value == format!("./{file_stem}.tsx")
                || source_value == format!("./{file_stem}.jsx"));

        if is_self_import {
            ctx.report(Diagnostic {
                rule_name: "import/no-cycle".to_owned(),
                message: "Module imports itself, creating a circular dependency".to_owned(),
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCycle)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_self_import() {
        let diags = lint_with_path(
            r#"import { foo } from "./myModule";"#,
            Path::new("myModule.ts"),
        );
        assert_eq!(diags.len(), 1, "self-import should be flagged as a cycle");
    }

    #[test]
    fn test_allows_different_module() {
        let diags = lint_with_path(
            r#"import { foo } from "./other";"#,
            Path::new("myModule.ts"),
        );
        assert!(
            diags.is_empty(),
            "importing a different module should not be flagged"
        );
    }

    #[test]
    fn test_allows_bare_specifier() {
        let diags = lint_with_path(r#"import { foo } from "lodash";"#, Path::new("myModule.ts"));
        assert!(diags.is_empty(), "bare specifier should not be flagged");
    }
}
