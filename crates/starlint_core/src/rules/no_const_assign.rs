//! Rule: `no-const-assign` (eslint)
//!
//! Disallow reassignment of `const` variables. Modifying a constant after
//! declaration causes a runtime error.

use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;
use oxc_ast::ast_kind::AstType;
use oxc_semantic::SymbolFlags;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags reassignment of `const` variables.
#[derive(Debug)]
pub struct NoConstAssign;

impl NativeRule for NoConstAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-const-assign".to_owned(),
            description: "Disallow reassignment of const variables".to_owned(),
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

        if decl.kind != VariableDeclarationKind::Const {
            return;
        }

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

                let flags = scoping.symbol_flags(symbol_id);
                if !flags.contains(SymbolFlags::ConstVariable) {
                    continue;
                }

                // Check if any reference to this symbol is a write
                let has_write = scoping
                    .get_resolved_references(symbol_id)
                    .any(oxc_semantic::Reference::is_write);

                if has_write {
                    ctx.report_error(
                        "no-const-assign",
                        &format!("'{}' is a constant and cannot be reassigned", binding.name),
                        Span::new(binding.span.start, binding.span.end),
                    );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstAssign)];
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
    fn test_flags_const_reassignment() {
        let diags = lint("const x = 1; x = 2;");
        assert_eq!(diags.len(), 1, "reassigning const should be flagged");
    }

    #[test]
    fn test_allows_const_read() {
        let diags = lint("const x = 1; console.log(x);");
        assert!(diags.is_empty(), "reading const should not be flagged");
    }

    #[test]
    fn test_allows_let_reassignment() {
        let diags = lint("let x = 1; x = 2;");
        assert!(diags.is_empty(), "reassigning let should not be flagged");
    }

    #[test]
    fn test_flags_const_destructuring_reassignment() {
        let diags = lint("const { a } = obj; a = 2;");
        assert_eq!(
            diags.len(),
            1,
            "reassigning destructured const should be flagged"
        );
    }
}
