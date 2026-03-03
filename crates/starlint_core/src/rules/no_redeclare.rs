//! Rule: `no-redeclare`
//!
//! Disallow variable redeclaration within the same scope.
//! Uses semantic analysis to detect when the same name is bound
//! multiple times in a single scope.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variables that are redeclared in the same scope.
#[derive(Debug)]
pub struct NoRedeclare;

impl NativeRule for NoRedeclare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-redeclare".to_owned(),
            description: "Disallow variable redeclaration".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::VariableDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclaration(decl) = kind else {
            return;
        };

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        for declarator in &decl.declarations {
            let binding_ids = declarator.id.get_binding_identifiers();

            for binding in &binding_ids {
                let Some(symbol_id) = binding.symbol_id.get() else {
                    continue;
                };

                // Check for redeclarations via the semantic redeclare list
                let redeclarations = scoping.symbol_redeclarations(symbol_id);
                if !redeclarations.is_empty() {
                    // Only report on the redeclaration, not the original
                    // The original declaration's span will differ from the redecl spans
                    for respan in redeclarations {
                        ctx.report_error(
                            "no-redeclare",
                            &format!("'{}' is already defined", binding.name),
                            Span::new(respan.span.start, respan.span.end),
                        );
                    }
                }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRedeclare)];
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
    fn test_flags_var_redeclaration() {
        let diags = lint("var x = 1; var x = 2;");
        assert!(!diags.is_empty(), "var redeclaration should be flagged");
    }

    #[test]
    fn test_allows_different_names() {
        let diags = lint("var x = 1; var y = 2;");
        assert!(diags.is_empty(), "different names should not be flagged");
    }

    #[test]
    fn test_allows_different_scopes() {
        let diags = lint("var x = 1; function foo() { var x = 2; }");
        assert!(
            diags.is_empty(),
            "different scopes should not be flagged by no-redeclare"
        );
    }

    #[test]
    fn test_allows_let_in_different_blocks() {
        let diags = lint("{ let x = 1; } { let x = 2; }");
        assert!(
            diags.is_empty(),
            "let in different blocks should not be flagged"
        );
    }
}
