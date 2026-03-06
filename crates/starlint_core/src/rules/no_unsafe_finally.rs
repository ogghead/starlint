//! Rule: `no-unsafe-finally`
//!
//! Disallow control flow statements in `finally` blocks. `return`, `throw`,
//! `break`, and `continue` in a `finally` block silently discard any exception
//! or return value from the `try`/`catch` blocks, leading to confusing behavior.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags control flow statements (`return`, `throw`, `break`, `continue`)
/// inside `finally` blocks.
#[derive(Debug)]
pub struct NoUnsafeFinally;

impl NativeRule for NoUnsafeFinally {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unsafe-finally".to_owned(),
            description: "Disallow control flow statements in finally blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TryStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TryStatement(try_stmt) = kind else {
            return;
        };

        let Some(finalizer) = &try_stmt.finalizer else {
            return;
        };

        check_statements_for_control_flow(&finalizer.body, ctx);
    }
}

/// Scan statements for control flow that would discard try/catch results.
fn check_statements_for_control_flow(stmts: &[Statement<'_>], ctx: &mut NativeLintContext<'_>) {
    for stmt in stmts {
        check_statement_for_control_flow(stmt, ctx);
    }
}

/// Check a single statement for unsafe control flow.
fn check_statement_for_control_flow(stmt: &Statement<'_>, ctx: &mut NativeLintContext<'_>) {
    match stmt {
        Statement::ReturnStatement(ret) => {
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `return` in finally block".to_owned(),
                span: Span::new(ret.span.start, ret.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Statement::ThrowStatement(throw) => {
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `throw` in finally block".to_owned(),
                span: Span::new(throw.span.start, throw.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Statement::BreakStatement(brk) => {
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `break` in finally block".to_owned(),
                span: Span::new(brk.span.start, brk.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Statement::ContinueStatement(cont) => {
            ctx.report(Diagnostic {
                rule_name: "no-unsafe-finally".to_owned(),
                message: "Unsafe `continue` in finally block".to_owned(),
                span: Span::new(cont.span.start, cont.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
        Statement::BlockStatement(block) => {
            check_statements_for_control_flow(&block.body, ctx);
        }
        Statement::IfStatement(if_stmt) => {
            check_statement_for_control_flow(&if_stmt.consequent, ctx);
            if let Some(alt) = &if_stmt.alternate {
                check_statement_for_control_flow(alt, ctx);
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeFinally)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_in_finally() {
        let diags = lint("try {} finally { return 1; }");
        assert_eq!(diags.len(), 1, "return in finally should be flagged");
    }

    #[test]
    fn test_flags_throw_in_finally() {
        let diags = lint("try {} finally { throw new Error(); }");
        assert_eq!(diags.len(), 1, "throw in finally should be flagged");
    }

    #[test]
    fn test_flags_break_in_finally() {
        let diags = lint("A: try {} finally { break A; }");
        assert_eq!(diags.len(), 1, "break in finally should be flagged");
    }

    #[test]
    fn test_allows_no_finally() {
        let diags = lint("try { return 1; } catch (e) {}");
        assert!(
            diags.is_empty(),
            "try without finally should not be flagged"
        );
    }

    #[test]
    fn test_allows_safe_finally() {
        let diags = lint("try {} finally { console.log('done'); }");
        assert!(diags.is_empty(), "safe finally should not be flagged");
    }

    #[test]
    fn test_allows_return_in_catch() {
        let diags = lint("try {} catch (e) { return 1; } finally {}");
        assert!(
            diags.is_empty(),
            "return in catch (not finally) should not be flagged"
        );
    }
}
