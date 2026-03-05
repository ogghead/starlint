//! Rule: `import/namespace`
//!
//! Validate namespace (star) imports. Namespace imports pull in everything
//! from a module and can mask unused dependencies. This rule flags namespace
//! imports from JSON modules (which only have a default export) as a
//! heuristic without full module resolution.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportDeclarationSpecifier;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags potentially invalid namespace imports.
#[derive(Debug)]
pub struct NamespaceImport;

impl NativeRule for NamespaceImport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/namespace".to_owned(),
            description: "Validate namespace (star) imports".to_owned(),
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

        // Type-only imports don't need runtime validation
        if import.import_kind.is_type() {
            return;
        }

        let Some(specifiers) = &import.specifiers else {
            return;
        };
        let has_namespace = specifiers.iter().any(|spec| {
            matches!(
                spec,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
            )
        });

        if !has_namespace {
            return;
        }

        let source_value = import.source.value.as_str();

        // Heuristic: namespace import from JSON makes no sense
        if std::path::Path::new(source_value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            ctx.report(Diagnostic {
                rule_name: "import/namespace".to_owned(),
                message: "Namespace import from JSON module is not useful (JSON modules only have a default export)".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NamespaceImport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_namespace_import_from_json() {
        let diags = lint(r#"import * as data from "./data.json";"#);
        assert_eq!(
            diags.len(),
            1,
            "namespace import from JSON should be flagged"
        );
    }

    #[test]
    fn test_allows_namespace_import_from_js() {
        let diags = lint(r#"import * as utils from "./utils";"#);
        assert!(
            diags.is_empty(),
            "namespace import from JS module should not be flagged"
        );
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "./module";"#);
        assert!(
            diags.is_empty(),
            "named import should not be flagged by namespace rule"
        );
    }
}
