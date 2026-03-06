//! Rule: `no-import-assign` (eslint)
//!
//! Disallow reassignment of imported bindings. Import bindings are
//! read-only; attempting to reassign them throws a runtime error.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportDeclarationSpecifier;
use oxc_ast::ast_kind::AstType;
use oxc_semantic::SymbolFlags;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags reassignment of imported bindings.
#[derive(Debug)]
pub struct NoImportAssign;

impl NativeRule for NoImportAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-import-assign".to_owned(),
            description: "Disallow reassignment of imported bindings".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
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

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        for specifier in specifiers {
            let local = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(s) => &s.local,
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => &s.local,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => &s.local,
            };

            let Some(symbol_id) = local.symbol_id.get() else {
                continue;
            };

            let flags = scoping.symbol_flags(symbol_id);
            if !flags.contains(SymbolFlags::Import) {
                continue;
            }

            // Check if any reference to this symbol is a write
            let has_write = scoping
                .get_resolved_references(symbol_id)
                .any(oxc_semantic::Reference::is_write);

            if has_write {
                ctx.report(Diagnostic {
                    rule_name: "no-import-assign".to_owned(),
                    message: format!(
                        "'{}' is an imported binding and cannot be reassigned",
                        local.name
                    ),
                    span: Span::new(local.span.start, local.span.end),
                    severity: Severity::Error,
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
    use crate::parser::{build_semantic, parse_file};
    use crate::traversal::traverse_and_lint_with_semantic;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let program = allocator.alloc(parsed.program);
            let semantic = build_semantic(program);
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoImportAssign)];
            traverse_and_lint_with_semantic(
                program,
                &rules,
                source,
                Path::new("test.js"),
                Some(&semantic),
            )
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_import_reassignment() {
        let diags = lint("import foo from 'bar'; foo = 1;");
        assert_eq!(diags.len(), 1, "reassigning import should be flagged");
    }

    #[test]
    fn test_allows_import_read() {
        let diags = lint("import foo from 'bar'; console.log(foo);");
        assert!(diags.is_empty(), "reading import should not be flagged");
    }

    #[test]
    fn test_flags_named_import_reassignment() {
        let diags = lint("import { foo } from 'bar'; foo = 1;");
        assert_eq!(diags.len(), 1, "reassigning named import should be flagged");
    }

    #[test]
    fn test_allows_import_call() {
        let diags = lint("import foo from 'bar'; foo();");
        assert!(diags.is_empty(), "calling import should not be flagged");
    }
}
