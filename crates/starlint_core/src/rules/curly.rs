//! Rule: `curly`
//!
//! Require braces around the body of control flow statements
//! (`if`, `else`, `for`, `while`, `do`). Omitting braces can lead
//! to bugs when adding statements to the body later.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags control flow statements whose body is not a block statement.
#[derive(Debug)]
pub struct Curly;

/// Check if a statement is a block statement (has curly braces).
const fn is_block(stmt: &Statement<'_>) -> bool {
    matches!(stmt, Statement::BlockStatement(_))
}

/// Get the span of a statement.
fn stmt_span(stmt: &Statement<'_>) -> oxc_span::Span {
    match stmt {
        Statement::ExpressionStatement(s) => s.span,
        Statement::ReturnStatement(s) => s.span,
        Statement::BreakStatement(s) => s.span,
        Statement::ContinueStatement(s) => s.span,
        Statement::ThrowStatement(s) => s.span,
        Statement::VariableDeclaration(s) => s.span,
        _ => stmt.span(),
    }
}

impl NativeRule for Curly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "curly".to_owned(),
            description: "Require curly braces for all control flow".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::IfStatement,
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::IfStatement(stmt) => {
                if !is_block(&stmt.consequent) {
                    let body = stmt_span(&stmt.consequent);
                    report_curly_fix(ctx, "Expected { after 'if' condition", stmt.span, body);
                }
                if let Some(alternate) = &stmt.alternate {
                    // Don't flag `else if` — only flag `else` without braces
                    if !is_block(alternate) && !matches!(alternate, Statement::IfStatement(_)) {
                        let body = stmt_span(alternate);
                        report_curly_fix(ctx, "Expected { after 'else'", stmt.span, body);
                    }
                }
            }
            AstKind::ForStatement(stmt) => {
                if !is_block(&stmt.body) {
                    let body = stmt_span(&stmt.body);
                    report_curly_fix(ctx, "Expected { after 'for'", stmt.span, body);
                }
            }
            AstKind::ForInStatement(stmt) => {
                if !is_block(&stmt.body) {
                    let body = stmt_span(&stmt.body);
                    report_curly_fix(ctx, "Expected { after 'for-in'", stmt.span, body);
                }
            }
            AstKind::ForOfStatement(stmt) => {
                if !is_block(&stmt.body) {
                    let body = stmt_span(&stmt.body);
                    report_curly_fix(ctx, "Expected { after 'for-of'", stmt.span, body);
                }
            }
            AstKind::WhileStatement(stmt) => {
                if !is_block(&stmt.body) {
                    let body = stmt_span(&stmt.body);
                    report_curly_fix(ctx, "Expected { after 'while' condition", stmt.span, body);
                }
            }
            AstKind::DoWhileStatement(stmt) => {
                if !is_block(&stmt.body) {
                    let body = stmt_span(&stmt.body);
                    report_curly_fix(ctx, "Expected { after 'do'", stmt.span, body);
                }
            }
            _ => {}
        }
    }
}

/// Report a curly-brace fix by wrapping the body statement in `{ ... }`.
fn report_curly_fix(
    ctx: &mut NativeLintContext<'_>,
    message: &str,
    stmt_span: oxc_span::Span,
    body_span: oxc_span::Span,
) {
    let source = ctx.source_text();
    let start = usize::try_from(body_span.start).unwrap_or(0);
    let end = usize::try_from(body_span.end).unwrap_or(start);
    let body_text = source.get(start..end).unwrap_or_default().to_owned();

    ctx.report(Diagnostic {
        rule_name: "curly".to_owned(),
        message: message.to_owned(),
        span: Span::new(stmt_span.start, stmt_span.end),
        severity: Severity::Warning,
        help: Some("Wrap in curly braces".to_owned()),
        fix: Some(Fix {
            message: "Wrap in curly braces".to_owned(),
            edits: vec![Edit {
                span: Span::new(body_span.start, body_span.end),
                replacement: format!("{{ {body_text} }}"),
            }],
        }),
        labels: vec![],
    });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Curly)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_if_without_braces() {
        let diags = lint("if (true) return;");
        assert_eq!(diags.len(), 1, "if without braces should be flagged");
    }

    #[test]
    fn test_allows_if_with_braces() {
        let diags = lint("if (true) { return; }");
        assert!(diags.is_empty(), "if with braces should not be flagged");
    }

    #[test]
    fn test_flags_else_without_braces() {
        let diags = lint("if (true) { return; } else return;");
        assert_eq!(diags.len(), 1, "else without braces should be flagged");
    }

    #[test]
    fn test_allows_else_if() {
        let diags = lint("if (a) { return; } else if (b) { return; }");
        assert!(diags.is_empty(), "else if should not be flagged");
    }

    #[test]
    fn test_flags_while_without_braces() {
        let diags = lint("while (true) break;");
        assert_eq!(diags.len(), 1, "while without braces should be flagged");
    }

    #[test]
    fn test_flags_for_without_braces() {
        let diags = lint("for (var i = 0; i < 10; i++) console.log(i);");
        assert_eq!(diags.len(), 1, "for without braces should be flagged");
    }
}
