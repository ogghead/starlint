//! Rule: `no-loop-func`
//!
//! Disallow function declarations and expressions inside loop statements.
//! Functions created in loops can lead to closure bugs where the loop
//! variable is shared.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags function declarations/expressions inside loops.
#[derive(Debug)]
pub struct NoLoopFunc;

impl NativeRule for NoLoopFunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-loop-func".to_owned(),
            description: "Disallow function declarations and expressions inside loops".to_owned(),
            category: Category::Style,
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
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Check loop bodies for function declarations
        let loop_body: Option<&Statement<'_>> = match kind {
            AstKind::ForStatement(stmt) => Some(&stmt.body),
            AstKind::ForInStatement(stmt) => Some(&stmt.body),
            AstKind::ForOfStatement(stmt) => Some(&stmt.body),
            AstKind::WhileStatement(stmt) => Some(&stmt.body),
            AstKind::DoWhileStatement(stmt) => Some(&stmt.body),
            _ => None,
        };

        let Some(body) = loop_body else {
            return;
        };

        // Check the direct body block for function declarations
        if let Statement::BlockStatement(block) = body {
            let mut spans: Vec<Span> = Vec::new();

            for stmt in &block.body {
                if let Statement::FunctionDeclaration(func) = stmt {
                    spans.push(Span::new(func.span.start, func.span.end));
                }
            }

            for span in spans {
                ctx.report_warning("no-loop-func", "Function declaration inside a loop", span);
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLoopFunc)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_function_in_for_loop() {
        let diags = lint("for (var i = 0; i < 10; i++) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in for loop should be flagged");
    }

    #[test]
    fn test_flags_function_in_while_loop() {
        let diags = lint("while (true) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in while loop should be flagged");
    }

    #[test]
    fn test_allows_function_outside_loop() {
        let diags = lint("function foo() {} for (var i = 0; i < 10; i++) {}");
        assert!(
            diags.is_empty(),
            "function outside loop should not be flagged"
        );
    }
}
