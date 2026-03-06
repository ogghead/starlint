//! Rule: `func-style`
//!
//! Enforce consistent function style. By default, prefers function declarations
//! over `const` function expressions. Arrow functions are allowed.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `const foo = function() {}` — prefer function declarations.
#[derive(Debug)]
pub struct FuncStyle;

impl NativeRule for FuncStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "func-style".to_owned(),
            description: "Enforce consistent use of function declarations vs expressions"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::VariableDeclarator])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclarator(decl) = kind else {
            return;
        };

        let Some(init) = &decl.init else {
            return;
        };

        // Only flag function expressions, not arrow functions
        if matches!(init, Expression::FunctionExpression(_)) {
            ctx.report(Diagnostic {
                rule_name: "func-style".to_owned(),
                message: "Use a function declaration instead of a const function expression"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(FuncStyle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_const_function_expression() {
        let diags = lint("const foo = function() {};");
        assert_eq!(
            diags.len(),
            1,
            "const function expression should be flagged"
        );
    }

    #[test]
    fn test_flags_named_function_expression() {
        let diags = lint("const foo = function bar() {};");
        assert_eq!(
            diags.len(),
            1,
            "named const function expression should be flagged"
        );
    }

    #[test]
    fn test_allows_function_declaration() {
        let diags = lint("function foo() {}");
        assert!(
            diags.is_empty(),
            "function declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_arrow_function() {
        let diags = lint("const foo = () => {};");
        assert!(diags.is_empty(), "arrow function should not be flagged");
    }

    #[test]
    fn test_allows_non_function_init() {
        let diags = lint("const foo = 42;");
        assert!(diags.is_empty(), "numeric assignment should not be flagged");
    }
}
