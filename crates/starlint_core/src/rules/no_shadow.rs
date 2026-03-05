//! Rule: `no-shadow`
//!
//! Disallow variable declarations from shadowing variables declared in an
//! outer scope. Shadowing can lead to confusion about which variable is
//! being referenced.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;
use oxc_span::Ident;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations that shadow a variable from an outer scope.
#[derive(Debug)]
pub struct NoShadow;

impl NativeRule for NoShadow {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-shadow".to_owned(),
            description: "Disallow variable declarations from shadowing variables in outer scope"
                .to_owned(),
            category: Category::Suggestion,
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

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        for declarator in &decl.declarations {
            let binding_ids = declarator.id.get_binding_identifiers();

            for binding in &binding_ids {
                let name = binding.name.as_str();
                let Some(symbol_id) = binding.symbol_id.get() else {
                    continue;
                };

                // Get the scope of this binding
                let binding_scope = scoping.symbol_scope_id(symbol_id);

                // Walk up parent scopes looking for a same-named binding
                let ident = Ident::from(name);
                let mut current_scope = scoping.scope_parent_id(binding_scope);

                while let Some(scope_id) = current_scope {
                    if scoping.get_binding(scope_id, ident).is_some() {
                        ctx.report(Diagnostic {
                            rule_name: "no-shadow".to_owned(),
                            message: format!("'{name}' is already declared in the upper scope"),
                            span: Span::new(binding.span.start, binding.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                        break;
                    }

                    current_scope = scoping.scope_parent_id(scope_id);
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoShadow)];
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
    fn test_flags_shadowed_var() {
        let diags = lint("var x = 1; function foo() { var x = 2; }");
        assert_eq!(diags.len(), 1, "shadowed var should be flagged");
    }

    #[test]
    fn test_flags_shadowed_let() {
        let diags = lint("let x = 1; { let x = 2; }");
        assert_eq!(diags.len(), 1, "shadowed let should be flagged");
    }

    #[test]
    fn test_allows_different_names() {
        let diags = lint("var x = 1; function foo() { var y = 2; }");
        assert!(diags.is_empty(), "different names should not be flagged");
    }

    #[test]
    fn test_allows_same_scope() {
        // Same-scope redeclaration is handled by no-redeclare, not no-shadow
        let diags = lint("var x = 1; var y = 2;");
        assert!(diags.is_empty(), "same scope should not be flagged");
    }

    #[test]
    fn test_nested_shadow() {
        let diags = lint("var x = 1; function foo() { var x = 2; function bar() { var x = 3; } }");
        assert_eq!(diags.len(), 2, "each nested shadow should be flagged");
    }
}
