//! Rule: `jest/prefer-todo`
//!
//! Suggest `test.todo('title')` for empty test cases. An empty test body
//! (or one with no assertions) is likely a placeholder; using `test.todo`
//! makes the intent explicit and appears in test runner summaries.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, FunctionBody, Statement};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags empty `it()` / `test()` callbacks that should use `test.todo()`.
#[derive(Debug)]
pub struct PreferTodo;

impl NativeRule for PreferTodo {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-todo".to_owned(),
            description: "Suggest using `test.todo()` for empty test cases".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `it(...)` or `test(...)`
        let callee_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };
        if callee_name != "it" && callee_name != "test" {
            return;
        }

        // Must have at least 2 arguments (title, callback)
        if call.arguments.len() < 2 {
            return;
        }
        let Some(second_arg) = call.arguments.get(1) else {
            return;
        };
        let Some(callback_expr) = second_arg.as_expression() else {
            return;
        };

        // Check if the callback has an empty body
        let is_empty = match callback_expr {
            Expression::ArrowFunctionExpression(arrow) => is_body_empty(&arrow.body),
            Expression::FunctionExpression(func) => {
                func.body.as_ref().is_some_and(|b| is_body_empty(b))
            }
            _ => false,
        };

        if is_empty {
            let source = ctx.source_text();
            let fix = call.arguments.first().map(|a| {
                let sp = a.span();
                let title = source[sp.start as usize..sp.end as usize].to_owned();
                let replacement = format!("{callee_name}.todo({title})");
                Fix {
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }
            });

            ctx.report(Diagnostic {
                rule_name: "jest/prefer-todo".to_owned(),
                message: "Use `test.todo()` instead of an empty test callback".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `test.todo()`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if a function body is empty (no statements, or only empty statements).
fn is_body_empty(body: &FunctionBody<'_>) -> bool {
    if body.statements.is_empty() {
        return true;
    }
    // Also treat bodies with only empty statements as empty
    body.statements
        .iter()
        .all(|s| matches!(s, Statement::EmptyStatement(_)))
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferTodo)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_arrow_test() {
        let diags = lint("test('should work', () => {});");
        assert_eq!(diags.len(), 1, "empty arrow test should be flagged");
    }

    #[test]
    fn test_flags_empty_function_test() {
        let diags = lint("it('should work', function() {});");
        assert_eq!(diags.len(), 1, "empty function test should be flagged");
    }

    #[test]
    fn test_allows_test_with_body() {
        let diags = lint("test('should work', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "test with body should not be flagged");
    }
}
