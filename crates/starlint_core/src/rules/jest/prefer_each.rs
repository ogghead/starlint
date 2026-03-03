//! Rule: `jest/prefer-each`
//!
//! Suggest `test.each` over repeated similar tests. When a `describe` block
//! contains 3 or more `it`/`test` calls with titles sharing a common prefix,
//! the test suite would benefit from parameterization via `test.each`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `describe` blocks with 3+ similarly-titled test cases.
#[derive(Debug)]
pub struct PreferEach;

/// Minimum number of tests with the same prefix to trigger the rule.
const MIN_SIMILAR_TESTS: usize = 3;

/// Minimum prefix length (in chars) to consider titles "similar".
const MIN_PREFIX_LEN: usize = 5;

impl NativeRule for PreferEach {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-each".to_owned(),
            description: "Suggest using `test.each` over repeated similar tests".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `describe(...)` call
        let is_describe = matches!(
            &call.callee,
            Expression::Identifier(id) if id.name.as_str() == "describe"
        );
        if !is_describe {
            return;
        }

        // Second argument should be the callback
        let Some(second_arg) = call.arguments.get(1) else {
            return;
        };
        let Some(callback_expr) = second_arg.as_expression() else {
            return;
        };

        // Get the function body
        let body = match callback_expr {
            Expression::ArrowFunctionExpression(arrow) => &arrow.body,
            Expression::FunctionExpression(func) => {
                let Some(ref body) = func.body else {
                    return;
                };
                body
            }
            _ => return,
        };

        // Collect test titles from top-level `it`/`test` calls in the body
        let mut titles: Vec<&str> = Vec::new();
        for stmt in &body.statements {
            let Statement::ExpressionStatement(expr_stmt) = stmt else {
                continue;
            };
            let Expression::CallExpression(inner_call) = &expr_stmt.expression else {
                continue;
            };
            let is_test = matches!(
                &inner_call.callee,
                Expression::Identifier(id) if id.name.as_str() == "it" || id.name.as_str() == "test"
            );
            if !is_test {
                continue;
            }
            if let Some(Argument::StringLiteral(s)) = inner_call.arguments.first() {
                titles.push(s.value.as_str());
            }
        }

        if titles.len() < MIN_SIMILAR_TESTS {
            return;
        }

        // Check if titles share a common prefix of meaningful length
        if let Some(first) = titles.first() {
            let prefix_len = titles.iter().skip(1).fold(first.len(), |acc, title| {
                let common = first
                    .chars()
                    .zip(title.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
                if common < acc { common } else { acc }
            });

            if prefix_len >= MIN_PREFIX_LEN {
                ctx.report_warning(
                    "jest/prefer-each",
                    "Consider using `test.each` to parameterize these similar test cases",
                    Span::new(call.span.start, call.span.end),
                );
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferEach)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_similar_tests() {
        let source = r"
describe('math', () => {
    test('handles addition with 1', () => {});
    test('handles addition with 2', () => {});
    test('handles addition with 3', () => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "3+ tests with similar titles should be flagged"
        );
    }

    #[test]
    fn test_allows_different_titles() {
        let source = r"
describe('math', () => {
    test('adds numbers', () => {});
    test('subtracts numbers', () => {});
    test('multiplies numbers', () => {});
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "tests with different titles should not be flagged"
        );
    }

    #[test]
    fn test_allows_few_tests() {
        let source = r"
describe('math', () => {
    test('handles input 1', () => {});
    test('handles input 2', () => {});
});
";
        let diags = lint(source);
        assert!(diags.is_empty(), "fewer than 3 tests should not be flagged");
    }
}
