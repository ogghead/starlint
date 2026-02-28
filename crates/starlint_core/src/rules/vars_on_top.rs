//! Rule: `vars-on-top`
//!
//! Require `var` declarations to be at the top of their enclosing scope
//! (function body or program). This ensures all `var`s are declared before
//! any other statements, making hoisting behavior explicit.

use oxc_ast::AstKind;
use oxc_ast::ast::{Statement, VariableDeclarationKind};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `var` declarations that are not at the top of their scope.
#[derive(Debug)]
pub struct VarsOnTop;

/// Check whether a statement is a var declaration or a directive ("use strict" etc).
fn is_var_or_directive(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::VariableDeclaration(decl) => decl.kind == VariableDeclarationKind::Var,
        Statement::ExpressionStatement(expr) => {
            // Directive prologues (e.g. "use strict") are string literal expression statements
            matches!(&expr.expression, oxc_ast::ast::Expression::StringLiteral(_))
        }
        _ => false,
    }
}

impl NativeRule for VarsOnTop {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "vars-on-top".to_owned(),
            description: "Require var declarations to be at the top of their scope".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // We check at the scope level (Program or FunctionBody) to see if
        // var declarations come before any non-var statements.
        match kind {
            AstKind::Program(program) => {
                check_statements(&program.body, ctx);
            }
            AstKind::FunctionBody(body) => {
                check_statements(&body.statements, ctx);
            }
            _ => {}
        }
    }
}

/// Check a list of statements for var declarations that appear after non-var statements.
fn check_statements(stmts: &[Statement<'_>], ctx: &mut NativeLintContext<'_>) {
    let mut found_non_var = false;

    for stmt in stmts {
        if found_non_var {
            // Any var declaration after a non-var statement is a violation
            if let Statement::VariableDeclaration(decl) = stmt {
                if decl.kind == VariableDeclarationKind::Var {
                    ctx.report_warning(
                        "vars-on-top",
                        "All `var` declarations must be at the top of the scope",
                        Span::new(decl.span.start, decl.span.end),
                    );
                }
            }
        } else if !is_var_or_directive(stmt) {
            // Also allow import/export at top level before var
            let is_module_decl = matches!(
                stmt,
                Statement::ImportDeclaration(_)
                    | Statement::ExportAllDeclaration(_)
                    | Statement::ExportDefaultDeclaration(_)
                    | Statement::ExportNamedDeclaration(_)
            );
            if !is_module_decl {
                found_non_var = true;
            }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(VarsOnTop)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_var_at_top() {
        let diags = lint("var x = 1;\nvar y = 2;\nconsole.log(x);");
        assert!(diags.is_empty(), "vars at top should not be flagged");
    }

    #[test]
    fn test_flags_var_after_statement() {
        let diags = lint("console.log('hi');\nvar x = 1;");
        assert_eq!(
            diags.len(),
            1,
            "var after non-var statement should be flagged"
        );
    }

    #[test]
    fn test_allows_directive_before_var() {
        let diags = lint("\"use strict\";\nvar x = 1;");
        assert!(diags.is_empty(), "directive before var should be allowed");
    }

    #[test]
    fn test_allows_let_const_anywhere() {
        // This rule only applies to var, not let/const
        let diags = lint("console.log('hi');\nlet x = 1;\nconst y = 2;");
        assert!(diags.is_empty(), "let/const should not be checked");
    }

    #[test]
    fn test_flags_var_in_function() {
        let diags = lint("function foo() {\n  console.log('hi');\n  var x = 1;\n}");
        assert_eq!(
            diags.len(),
            1,
            "var after statement in function should be flagged"
        );
    }

    #[test]
    fn test_allows_var_at_top_of_function() {
        let diags = lint("function foo() {\n  var x = 1;\n  console.log(x);\n}");
        assert!(diags.is_empty(), "var at top of function should be allowed");
    }

    #[test]
    fn test_multiple_violations() {
        let diags = lint("console.log('a');\nvar x = 1;\nvar y = 2;");
        assert_eq!(diags.len(), 2, "each var after non-var should be flagged");
    }

    #[test]
    fn test_allows_import_before_var() {
        // Module syntax: imports typically come before vars
        let diags = lint("import foo from 'foo';\nvar x = 1;");
        assert!(diags.is_empty(), "import before var should be allowed");
    }
}
