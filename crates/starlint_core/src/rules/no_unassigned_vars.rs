//! Rule: `no-unassigned-vars`
//!
//! Flag `let` declarations without initializers. A `let x;` declaration
//! leaves the variable as `undefined` and is often a sign of incomplete
//! code. Prefer `let x = <value>;` or use `const` when possible.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, VariableDeclarationKind};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `let` declarators that have no initializer.
#[derive(Debug)]
pub struct NoUnassignedVars;

impl NativeRule for NoUnassignedVars {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unassigned-vars".to_owned(),
            description: "Disallow `let` declarations without an initializer".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclaration(decl) = kind else {
            return;
        };

        // Only flag `let` declarations — `var` is legacy, `const` requires an init.
        if decl.kind != VariableDeclarationKind::Let {
            return;
        }

        for declarator in &decl.declarations {
            // Skip if there is an initializer
            if declarator.init.is_some() {
                continue;
            }

            // Only flag simple binding identifiers (not destructured patterns)
            let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
                continue;
            };

            let name = ident.name.as_str();

            ctx.report_warning(
                "no-unassigned-vars",
                &format!("Variable `{name}` is declared with `let` but has no initializer"),
                Span::new(declarator.span.start, declarator.span.end),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnassignedVars)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_let_without_init() {
        let diags = lint("let x;");
        assert_eq!(diags.len(), 1, "let without initializer should be flagged");
    }

    #[test]
    fn test_allows_let_with_init() {
        let diags = lint("let x = 1;");
        assert!(
            diags.is_empty(),
            "let with initializer should not be flagged"
        );
    }

    #[test]
    fn test_allows_var_without_init() {
        let diags = lint("var x;");
        assert!(
            diags.is_empty(),
            "var without initializer should not be flagged (only checks let)"
        );
    }

    #[test]
    fn test_allows_const_with_init() {
        let diags = lint("const x = 1;");
        assert!(
            diags.is_empty(),
            "const with initializer should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_declarators() {
        let diags = lint("let a, b;");
        assert_eq!(
            diags.len(),
            2,
            "both declarators without initializers should be flagged"
        );
    }

    #[test]
    fn test_flags_only_uninitialised() {
        let diags = lint("let a = 1, b;");
        assert_eq!(
            diags.len(),
            1,
            "only the declarator without init should be flagged"
        );
    }

    #[test]
    fn test_allows_destructured_without_init() {
        // Destructured patterns without init are a syntax error in practice,
        // but the rule only flags simple identifiers — skip destructured.
        let diags = lint("let [a, b] = [1, 2];");
        assert!(
            diags.is_empty(),
            "destructured with init should not be flagged"
        );
    }
}
