//! Rule: `curly`
//!
//! Require braces around the body of control flow statements
//! (`if`, `else`, `for`, `while`, `do`). Omitting braces can lead
//! to bugs when adding statements to the body later.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags control flow statements whose body is not a block statement.
#[derive(Debug)]
pub struct Curly;

/// Check if a statement is a block statement (has curly braces).
const fn is_block(stmt: &Statement<'_>) -> bool {
    matches!(stmt, Statement::BlockStatement(_))
}

impl NativeRule for Curly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "curly".to_owned(),
            description: "Require curly braces for all control flow".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
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
                    ctx.report_warning(
                        "curly",
                        "Expected { after 'if' condition",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
                if let Some(alternate) = &stmt.alternate {
                    // Don't flag `else if` — only flag `else` without braces
                    if !is_block(alternate) && !matches!(alternate, Statement::IfStatement(_)) {
                        ctx.report_warning(
                            "curly",
                            "Expected { after 'else'",
                            Span::new(stmt.span.start, stmt.span.end),
                        );
                    }
                }
            }
            AstKind::ForStatement(stmt) => {
                if !is_block(&stmt.body) {
                    ctx.report_warning(
                        "curly",
                        "Expected { after 'for'",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstKind::ForInStatement(stmt) => {
                if !is_block(&stmt.body) {
                    ctx.report_warning(
                        "curly",
                        "Expected { after 'for-in'",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstKind::ForOfStatement(stmt) => {
                if !is_block(&stmt.body) {
                    ctx.report_warning(
                        "curly",
                        "Expected { after 'for-of'",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstKind::WhileStatement(stmt) => {
                if !is_block(&stmt.body) {
                    ctx.report_warning(
                        "curly",
                        "Expected { after 'while' condition",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstKind::DoWhileStatement(stmt) => {
                if !is_block(&stmt.body) {
                    ctx.report_warning(
                        "curly",
                        "Expected { after 'do'",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            _ => {}
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
