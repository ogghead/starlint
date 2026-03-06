//! Rule: `no-promise-executor-return`
//!
//! Disallow returning a value from a Promise executor function. The return
//! value of the executor is ignored, and returning a value is likely a mistake
//! (perhaps the author intended `resolve(value)` instead of `return value`).

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `return <value>` inside Promise executor functions.
#[derive(Debug)]
pub struct NoPromiseExecutorReturn;

impl NativeRule for NoPromiseExecutorReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-promise-executor-return".to_owned(),
            description: "Disallow returning a value from a Promise executor".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        // Check if this is `new Promise(...)`
        let Expression::Identifier(callee) = &new_expr.callee else {
            return;
        };

        if callee.name.as_str() != "Promise" {
            return;
        }

        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        // Get the function body from the executor
        match first_arg {
            oxc_ast::ast::Argument::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    check_statements_for_value_return(&body.statements, ctx);
                }
            }
            oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) => {
                check_statements_for_value_return(&arrow.body.statements, ctx);
            }
            _ => {}
        }
    }
}

/// Walk statements looking for return statements that have a value.
fn check_statements_for_value_return(stmts: &[Statement<'_>], ctx: &mut NativeLintContext<'_>) {
    for stmt in stmts {
        check_statement_for_value_return(stmt, ctx);
    }
}

/// Check a single statement for `return <value>`.
fn check_statement_for_value_return(stmt: &Statement<'_>, ctx: &mut NativeLintContext<'_>) {
    match stmt {
        Statement::ReturnStatement(ret) => {
            if ret.argument.is_some() {
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with bare `return;`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(ret.span.start, ret.span.end),
                        replacement: "return;".to_owned(),
                    }],
                    is_snippet: false,
                });
                ctx.report(Diagnostic {
                    rule_name: "no-promise-executor-return".to_owned(),
                    message: "Return statement in Promise executor is ignored".to_owned(),
                    span: Span::new(ret.span.start, ret.span.end),
                    severity: Severity::Error,
                    help: Some(
                        "Use `resolve(value)` or `reject(error)` instead of `return`".to_owned(),
                    ),
                    fix,
                    labels: vec![],
                });
            }
        }
        Statement::BlockStatement(block) => {
            check_statements_for_value_return(&block.body, ctx);
        }
        Statement::IfStatement(if_stmt) => {
            check_statement_for_value_return(&if_stmt.consequent, ctx);
            if let Some(alt) = &if_stmt.alternate {
                check_statement_for_value_return(alt, ctx);
            }
        }
        _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoPromiseExecutorReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_value_in_executor() {
        let diags = lint("new Promise(function(resolve, reject) { return 1; });");
        assert_eq!(diags.len(), 1, "return value in executor should be flagged");
    }

    #[test]
    fn test_flags_return_value_in_arrow_executor() {
        let diags = lint("new Promise((resolve, reject) => { return 1; });");
        assert_eq!(
            diags.len(),
            1,
            "return value in arrow executor should be flagged"
        );
    }

    #[test]
    fn test_allows_bare_return() {
        let diags = lint("new Promise(function(resolve, reject) { resolve(1); return; });");
        assert!(
            diags.is_empty(),
            "bare return in executor should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_return() {
        let diags = lint("new Promise(function(resolve, reject) { resolve(1); });");
        assert!(
            diags.is_empty(),
            "executor without return should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_promise() {
        let diags = lint("new Foo(function() { return 1; });");
        assert!(
            diags.is_empty(),
            "non-Promise constructor should not be flagged"
        );
    }
}
