//! Rule: `no-case-declarations`
//!
//! Disallow lexical declarations in case/default clauses. Lexical declarations
//! (`let`, `const`, `class`, `function`) in case clauses are visible to the
//! entire switch block but only initialized when the case is reached, which
//! can lead to unexpected errors.

use oxc_ast::AstKind;
use oxc_ast::ast::{Statement, VariableDeclarationKind};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags lexical declarations (`let`, `const`, `class`, `function`) inside
/// `case` / `default` clauses that are not wrapped in a block.
#[derive(Debug)]
pub struct NoCaseDeclarations;

impl NativeRule for NoCaseDeclarations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-case-declarations".to_owned(),
            description: "Disallow lexical declarations in case/default clauses".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SwitchCase])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SwitchCase(case) = kind else {
            return;
        };

        for stmt in &case.consequent {
            if let Some(span) = lexical_declaration_span(stmt) {
                ctx.report_error(
                    "no-case-declarations",
                    "Unexpected lexical declaration in case clause",
                    span,
                );
            }
        }
    }
}

/// If the statement is a lexical declaration, return its span.
fn lexical_declaration_span(stmt: &Statement<'_>) -> Option<Span> {
    match stmt {
        Statement::VariableDeclaration(decl)
            if decl.kind == VariableDeclarationKind::Let
                || decl.kind == VariableDeclarationKind::Const =>
        {
            Some(Span::new(decl.span.start, decl.span.end))
        }
        Statement::FunctionDeclaration(func) => Some(Span::new(func.span.start, func.span.end)),
        Statement::ClassDeclaration(class) => Some(Span::new(class.span.start, class.span.end)),
        _ => None,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCaseDeclarations)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_let_in_case() {
        let diags = lint("switch (x) { case 0: let y = 1; break; }");
        assert_eq!(diags.len(), 1, "let in case should be flagged");
    }

    #[test]
    fn test_flags_const_in_case() {
        let diags = lint("switch (x) { case 0: const y = 1; break; }");
        assert_eq!(diags.len(), 1, "const in case should be flagged");
    }

    #[test]
    fn test_flags_function_in_case() {
        let diags = lint("switch (x) { case 0: function f() {} break; }");
        assert_eq!(diags.len(), 1, "function decl in case should be flagged");
    }

    #[test]
    fn test_flags_class_in_case() {
        let diags = lint("switch (x) { case 0: class C {} break; }");
        assert_eq!(diags.len(), 1, "class decl in case should be flagged");
    }

    #[test]
    fn test_allows_var_in_case() {
        let diags = lint("switch (x) { case 0: var y = 1; break; }");
        assert!(diags.is_empty(), "var in case should not be flagged");
    }

    #[test]
    fn test_allows_let_in_block() {
        let diags = lint("switch (x) { case 0: { let y = 1; break; } }");
        assert!(
            diags.is_empty(),
            "let inside block in case should not be flagged"
        );
    }

    #[test]
    fn test_flags_in_default() {
        let diags = lint("switch (x) { default: let y = 1; }");
        assert_eq!(diags.len(), 1, "let in default should be flagged");
    }
}
