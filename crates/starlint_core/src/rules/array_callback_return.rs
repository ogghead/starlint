//! Rule: `array-callback-return`
//!
//! Enforce `return` statements in callbacks of array methods. Methods like
//! `map`, `filter`, `reduce`, `find`, `every`, `some`, `sort`, `flatMap`,
//! and `findIndex` expect their callbacks to return a value. Forgetting
//! to return is a common bug.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Array methods whose callbacks must return a value.
const METHODS_REQUIRING_RETURN: &[&str] = &[
    "map",
    "filter",
    "reduce",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "every",
    "some",
    "sort",
    "flatMap",
];

/// Flags callbacks in array methods that don't return a value.
#[derive(Debug)]
pub struct ArrayCallbackReturn;

impl NativeRule for ArrayCallbackReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "array-callback-return".to_owned(),
            description: "Enforce `return` in callbacks of array methods".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if callee is `something.map(...)`, `something.filter(...)`, etc.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();
        if !METHODS_REQUIRING_RETURN.contains(&method_name) {
            return;
        }

        // Check the first argument (the callback)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        match first_arg {
            Argument::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    if !statements_contain_return(&body.statements) {
                        ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                            rule_name: "array-callback-return".to_owned(),
                            message: format!(
                                "Expected a return value in `.{method_name}()` callback"
                            ),
                            span: Span::new(func.span.start, func.span.end),
                            severity: Severity::Error,
                            help: Some(format!(
                                "Array `.{method_name}()` expects a return value from its callback"
                            )),
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            Argument::ArrowFunctionExpression(arrow) => {
                // Arrow functions with expression bodies always return
                if arrow.expression {
                    return;
                }
                if !statements_contain_return(&arrow.body.statements) {
                    ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                        rule_name: "array-callback-return".to_owned(),
                        message: format!("Expected a return value in `.{method_name}()` callback"),
                        span: Span::new(arrow.span.start, arrow.span.end),
                        severity: Severity::Error,
                        help: Some(format!(
                            "Array `.{method_name}()` expects a return value from its callback"
                        )),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Check if statements contain a return statement with a value.
fn statements_contain_return(stmts: &[Statement<'_>]) -> bool {
    stmts.iter().any(|s| statement_contains_return(s))
}

/// Recursively check a statement for a return with a value.
fn statement_contains_return(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(ret) => ret.argument.is_some(),
        Statement::BlockStatement(block) => statements_contain_return(&block.body),
        Statement::IfStatement(if_stmt) => {
            statement_contains_return(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_ref()
                    .is_some_and(|alt| statement_contains_return(alt))
        }
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ArrayCallbackReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_map_without_return() {
        let diags = lint("[1,2,3].map(function(x) { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            "map callback without return should be flagged"
        );
    }

    #[test]
    fn test_allows_map_with_return() {
        let diags = lint("[1,2,3].map(function(x) { return x * 2; });");
        assert!(
            diags.is_empty(),
            "map callback with return should not be flagged"
        );
    }

    #[test]
    fn test_allows_arrow_expression() {
        let diags = lint("[1,2,3].map(x => x * 2);");
        assert!(
            diags.is_empty(),
            "arrow expression callback should not be flagged"
        );
    }

    #[test]
    fn test_flags_filter_without_return() {
        let diags = lint("[1,2,3].filter(function(x) { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            "filter callback without return should be flagged"
        );
    }

    #[test]
    fn test_allows_for_each() {
        let diags = lint("[1,2,3].forEach(function(x) { console.log(x); });");
        assert!(diags.is_empty(), "forEach callback should not be flagged");
    }

    #[test]
    fn test_flags_arrow_block_without_return() {
        let diags = lint("[1,2,3].map(x => { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            "arrow block callback without return should be flagged"
        );
    }
}
