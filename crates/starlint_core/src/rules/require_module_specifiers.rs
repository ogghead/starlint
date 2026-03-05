//! Rule: `require-module-specifiers`
//!
//! Flag import declarations that have no specifiers (side-effect imports like
//! `import 'foo'`). While sometimes needed for polyfills and CSS, they should
//! be used sparingly and intentionally.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportOrExportKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags side-effect-only imports that have no specifiers.
#[derive(Debug)]
pub struct RequireModuleSpecifiers;

impl NativeRule for RequireModuleSpecifiers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-module-specifiers".to_owned(),
            description: "Require import declarations to have specifiers".to_owned(),
            category: Category::Suggestion,
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

        // Allow `import type` statements (TypeScript type-only imports)
        if import.import_kind == ImportOrExportKind::Type {
            return;
        }

        // `specifiers` is `None` for `import 'foo'` (bare side-effect import)
        // and `Some(vec)` for imports with specifiers (possibly empty for `import {} from 'foo'`)
        let is_side_effect = match &import.specifiers {
            None => true,
            Some(specs) => specs.is_empty(),
        };

        if is_side_effect {
            let source = import.source.value.as_str();
            ctx.report(Diagnostic {
                rule_name: "require-module-specifiers".to_owned(),
                message: format!("Import from '{source}' has no specifiers — side-effect imports should be used sparingly"),
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

    /// Helper to lint source code as an ES module.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.mjs")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireModuleSpecifiers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.mjs"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bare_side_effect_import() {
        let diags = lint("import 'foo';");
        assert_eq!(diags.len(), 1, "bare side-effect import should be flagged");
    }

    #[test]
    fn test_flags_polyfill_import() {
        let diags = lint("import './polyfill';");
        assert_eq!(
            diags.len(),
            1,
            "polyfill side-effect import should be flagged"
        );
    }

    #[test]
    fn test_allows_default_import() {
        let diags = lint("import foo from 'foo';");
        assert!(diags.is_empty(), "default import should not be flagged");
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint("import { foo } from 'foo';");
        assert!(diags.is_empty(), "named import should not be flagged");
    }

    #[test]
    fn test_allows_namespace_import() {
        let diags = lint("import * as foo from 'foo';");
        assert!(diags.is_empty(), "namespace import should not be flagged");
    }

    #[test]
    fn test_allows_type_import() {
        let diags = lint("import type { Foo } from 'foo';");
        assert!(diags.is_empty(), "type-only import should not be flagged");
    }
}
