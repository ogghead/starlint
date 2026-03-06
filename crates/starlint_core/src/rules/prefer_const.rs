//! Rule: `prefer-const`
//!
//! Require `const` for variables that are never reassigned.
//! Uses semantic analysis to check whether each binding declared with `let`
//! has any write references. If none do, the declaration could use `const`.

use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `let` declarations where no binding is ever reassigned.
#[derive(Debug)]
pub struct PreferConst;

impl NativeRule for PreferConst {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-const".to_owned(),
            description: "Require `const` for variables that are never reassigned".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
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

        // Only flag `let` declarations.
        if decl.kind != VariableDeclarationKind::Let {
            return;
        }

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        // Check every declarator: each must have an initializer, and none of
        // its bindings may be written to after declaration.
        let all_const_eligible = decl.declarations.iter().all(|declarator| {
            // Without an initializer (`let x;`), it can't become `const`.
            if declarator.init.is_none() {
                return false;
            }

            let binding_ids = declarator.id.get_binding_identifiers();
            if binding_ids.is_empty() {
                return false;
            }

            binding_ids.iter().all(|binding| {
                let Some(symbol_id) = binding.symbol_id.get() else {
                    return false;
                };

                // If any resolved reference writes to this symbol, it's not const-eligible.
                !scoping
                    .get_resolved_references(symbol_id)
                    .any(oxc_semantic::Reference::is_write)
            })
        });

        if all_const_eligible && !decl.declarations.is_empty() {
            let let_span = Span::new(decl.span.start, decl.span.start.saturating_add(3));

            ctx.report(Diagnostic {
                rule_name: "prefer-const".to_owned(),
                message: "'let' declaration can use 'const' since variables are never reassigned"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: Some("Replace `let` with `const`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace `let` with `const`".to_owned(),
                    edits: vec![Edit {
                        span: let_span,
                        replacement: "const".to_owned(),
                    }],
                    is_snippet: false,
                }),
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

    /// Helper to lint source code with semantic analysis.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let program = allocator.alloc(parsed.program);
            let semantic = build_semantic(program);
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferConst)];
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
    fn test_let_with_no_reassignment() {
        let diags = lint("let x = 1;");
        assert_eq!(diags.len(), 1, "should flag let that is never reassigned");
        assert!(
            diags.first().is_some_and(|d| d.rule_name == "prefer-const"),
            "diagnostic should be from prefer-const"
        );
    }

    #[test]
    fn test_let_with_reassignment() {
        let diags = lint("let x = 1; x = 2;");
        assert!(diags.is_empty(), "should not flag let that is reassigned");
    }

    #[test]
    fn test_const_not_flagged() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "should not flag const declarations");
    }

    #[test]
    fn test_let_without_init() {
        let diags = lint("let x;");
        assert!(diags.is_empty(), "should not flag let without initializer");
    }

    #[test]
    fn test_let_without_init_then_assigned() {
        let diags = lint("let x; x = 1;");
        assert!(
            diags.is_empty(),
            "should not flag let without initializer even if only assigned once"
        );
    }

    #[test]
    fn test_var_not_flagged() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "should not flag var declarations");
    }

    #[test]
    fn test_multiple_declarators_all_const() {
        let diags = lint("let a = 1, b = 2;");
        assert_eq!(
            diags.len(),
            1,
            "should flag when all declarators are const-eligible"
        );
    }

    #[test]
    fn test_multiple_declarators_one_reassigned() {
        let diags = lint("let a = 1, b = 2; b = 3;");
        assert!(
            diags.is_empty(),
            "should not flag when any declarator is reassigned"
        );
    }

    #[test]
    fn test_destructuring_const_eligible() {
        let diags = lint("let { a, b } = obj;");
        assert_eq!(
            diags.len(),
            1,
            "should flag destructuring when no binding is reassigned"
        );
    }

    #[test]
    fn test_destructuring_reassigned() {
        let diags = lint("let { a, b } = obj; a = 2;");
        assert!(
            diags.is_empty(),
            "should not flag destructuring when any binding is reassigned"
        );
    }

    #[test]
    fn test_let_read_only() {
        let diags = lint("let x = 1; console.log(x);");
        assert_eq!(
            diags.len(),
            1,
            "should flag let that is only read, never written"
        );
    }

    #[test]
    fn test_let_increment() {
        let diags = lint("let x = 0; x++;");
        assert!(diags.is_empty(), "should not flag let that is incremented");
    }
}
