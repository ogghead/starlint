//! Rule: `no-unreachable`
//!
//! Disallow unreachable code after `return`, `throw`, `break`, or `continue`.
//! Code after these statements can never execute and is almost always a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags statements that appear after `return`, `throw`, `break`, or `continue`
/// within the same block.
#[derive(Debug)]
pub struct NoUnreachable;

impl NativeRule for NoUnreachable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unreachable".to_owned(),
            description: "Disallow unreachable code after return, throw, break, or continue"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::BlockStatement,
            AstType::FunctionBody,
            AstType::Program,
            AstType::SwitchCase,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Check block statements for unreachable code
        let statements: Option<&[Statement<'_>]> = match kind {
            AstKind::FunctionBody(body) => Some(&body.statements),
            AstKind::BlockStatement(block) => Some(&block.body),
            AstKind::Program(program) => Some(&program.body),
            AstKind::SwitchCase(case) => Some(&case.consequent),
            _ => None,
        };

        let Some(stmts) = statements else {
            return;
        };

        let mut found_terminator = false;
        for stmt in stmts {
            if found_terminator {
                // Skip function/class declarations — they're hoisted
                if is_hoisted_declaration(stmt) {
                    continue;
                }
                // Skip empty statements
                if matches!(stmt, Statement::EmptyStatement(_)) {
                    continue;
                }
                let span = statement_span(stmt);
                let fix = Some(Fix {
                    message: "Remove unreachable code".to_owned(),
                    edits: vec![Edit {
                        span,
                        replacement: String::new(),
                    }],
                });
                ctx.report(Diagnostic {
                    rule_name: "no-unreachable".to_owned(),
                    message: "Unreachable code".to_owned(),
                    span,
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
                // Only report the first unreachable statement per block
                break;
            }

            if is_terminator(stmt) {
                found_terminator = true;
            }
        }
    }
}

/// Check if a statement terminates execution flow.
const fn is_terminator(stmt: &Statement<'_>) -> bool {
    matches!(
        stmt,
        Statement::ReturnStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
    )
}

/// Check if a statement is a hoisted declaration (function/class).
const fn is_hoisted_declaration(stmt: &Statement<'_>) -> bool {
    matches!(stmt, Statement::FunctionDeclaration(_))
}

/// Get the span of a statement for reporting.
fn statement_span(stmt: &Statement<'_>) -> Span {
    use oxc_span::GetSpan;
    let span = stmt.span();
    Span::new(span.start, span.end)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnreachable)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_code_after_return() {
        let diags = lint("function f() { return 1; var x = 2; }");
        assert_eq!(diags.len(), 1, "code after return should be flagged");
    }

    #[test]
    fn test_flags_code_after_throw() {
        let diags = lint("function f() { throw new Error(); var x = 2; }");
        assert_eq!(diags.len(), 1, "code after throw should be flagged");
    }

    #[test]
    fn test_allows_code_before_return() {
        let diags = lint("function f() { var x = 1; return x; }");
        assert!(diags.is_empty(), "code before return should not be flagged");
    }

    #[test]
    fn test_allows_function_after_return() {
        // Function declarations are hoisted
        let diags = lint("function f() { return g(); function g() { return 1; } }");
        assert!(
            diags.is_empty(),
            "function declaration after return should not be flagged (hoisted)"
        );
    }

    #[test]
    fn test_allows_no_terminator() {
        let diags = lint("function f() { var x = 1; var y = 2; }");
        assert!(
            diags.is_empty(),
            "code without terminator should not be flagged"
        );
    }

    #[test]
    fn test_flags_code_after_break() {
        let diags = lint("for (;;) { break; var x = 1; }");
        assert_eq!(diags.len(), 1, "code after break should be flagged");
    }
}
