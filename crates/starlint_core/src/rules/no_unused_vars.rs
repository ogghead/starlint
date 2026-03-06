//! Rule: `no-unused-vars` (eslint)
//!
//! Disallow unused variables. Variables that are declared but never used
//! are most likely errors. This rule flags variables, functions, and
//! function parameters that are declared but never read.

use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags variables that are declared but never read.
#[derive(Debug)]
pub struct NoUnusedVars;

impl NativeRule for NoUnusedVars {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-vars".to_owned(),
            description: "Disallow unused variables".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
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

        // Skip `var` in for-in/for-of (often used as `for (var x of ...`)
        // We only check let/const/var top-level declarations
        if decl.kind == VariableDeclarationKind::Var {
            // Still check var, but be more lenient
        }

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        // Collect unused binding info (name, span) and count totals to decide
        // whether the entire declaration can be deleted.
        let mut total_bindings: usize = 0;
        let mut unused_infos: Vec<(String, Span)> = Vec::new();

        for declarator in &decl.declarations {
            let binding_ids = declarator.id.get_binding_identifiers();

            for binding in &binding_ids {
                // Skip variables starting with `_` (conventional "unused" marker)
                if binding.name.starts_with('_') {
                    continue;
                }

                total_bindings = total_bindings.saturating_add(1);

                let Some(symbol_id) = binding.symbol_id.get() else {
                    continue;
                };

                // Check if any reference to this symbol is a read
                let has_read = scoping
                    .get_resolved_references(symbol_id)
                    .any(oxc_semantic::Reference::is_read);

                if !has_read {
                    unused_infos.push((
                        binding.name.to_string(),
                        Span::new(binding.span.start, binding.span.end),
                    ));
                }
            }
        }

        // Only offer a fix to delete the declaration if ALL bindings are unused.
        let fix: Option<Fix> = if !unused_infos.is_empty() && unused_infos.len() == total_bindings {
            let decl_span = Span::new(decl.span.start, decl.span.end);
            FixBuilder::new("Remove unused declaration")
                .edit(fix_utils::delete_statement(ctx.source_text(), decl_span))
                .build()
        } else {
            None
        };

        for (name, span) in &unused_infos {
            ctx.report(Diagnostic {
                rule_name: "no-unused-vars".to_owned(),
                message: format!("'{name}' is declared but never used"),
                span: *span,
                severity: Severity::Warning,
                help: None,
                fix: fix.clone(),
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
    use crate::parser::{build_semantic, parse_file};
    use crate::traversal::traverse_and_lint_with_semantic;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let program = allocator.alloc(parsed.program);
            let semantic = build_semantic(program);
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnusedVars)];
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
    fn test_flags_unused_var() {
        let diags = lint("var x = 1;");
        assert_eq!(diags.len(), 1, "unused var should be flagged");
    }

    #[test]
    fn test_flags_unused_let() {
        let diags = lint("let x = 1;");
        assert_eq!(diags.len(), 1, "unused let should be flagged");
    }

    #[test]
    fn test_flags_unused_const() {
        let diags = lint("const x = 1;");
        assert_eq!(diags.len(), 1, "unused const should be flagged");
    }

    #[test]
    fn test_allows_used_variable() {
        let diags = lint("var x = 1; console.log(x);");
        assert!(diags.is_empty(), "used variable should not be flagged");
    }

    #[test]
    fn test_allows_underscore_prefix() {
        let diags = lint("var _x = 1;");
        assert!(
            diags.is_empty(),
            "underscore-prefixed variable should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_unused() {
        let diags = lint("var a = 1, b = 2;");
        assert_eq!(diags.len(), 2, "two unused vars should be flagged");
    }
}
