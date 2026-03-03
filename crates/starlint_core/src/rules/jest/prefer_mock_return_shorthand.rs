//! Rule: `jest/prefer-mock-return-shorthand`
//!
//! Suggest `jest.fn().mockReturnValue(x)` over
//! `jest.fn().mockImplementation(() => x)`. The shorthand is more readable
//! when the mock simply returns a static value.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.mockImplementation(() => x)` that could use `.mockReturnValue(x)`.
#[derive(Debug)]
pub struct PreferMockReturnShorthand;

impl NativeRule for PreferMockReturnShorthand {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-mock-return-shorthand".to_owned(),
            description:
                "Suggest using `mockReturnValue()` instead of `mockImplementation(() => x)`"
                    .to_owned(),
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

        // Must be `.mockImplementation(...)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name.as_str() != "mockImplementation" {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };
        let Some(arg_expr) = first_arg.as_expression() else {
            return;
        };

        // Check for arrow function with expression body and no parameters:
        // `() => x` (but NOT `() => Promise.resolve(x)` which is handled by
        // prefer-mock-promise-shorthand)
        let is_simple_return = match arg_expr {
            Expression::ArrowFunctionExpression(arrow) => {
                // Must be expression body with no parameters
                arrow.expression
                    && arrow.params.items.is_empty()
                    && !is_promise_call_in_body(&arrow.body)
            }
            _ => false,
        };

        if is_simple_return {
            ctx.report_warning(
                "jest/prefer-mock-return-shorthand",
                "Use `.mockReturnValue(x)` instead of `.mockImplementation(() => x)`",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if the arrow body contains a `Promise.resolve` or `Promise.reject` call.
fn is_promise_call_in_body(body: &oxc_ast::ast::FunctionBody<'_>) -> bool {
    body.statements.first().is_some_and(|stmt| {
        if let oxc_ast::ast::Statement::ExpressionStatement(es) = stmt {
            is_promise_call(&es.expression)
        } else {
            false
        }
    })
}

/// Check if an expression is `Promise.resolve(...)` or `Promise.reject(...)`.
fn is_promise_call(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    let Expression::Identifier(obj) = &member.object else {
        return false;
    };
    if obj.name.as_str() != "Promise" {
        return false;
    }
    let method = member.property.name.as_str();
    method == "resolve" || method == "reject"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferMockReturnShorthand)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_simple_arrow_return() {
        let diags = lint("jest.fn().mockImplementation(() => 42);");
        assert_eq!(
            diags.len(),
            1,
            "`mockImplementation(() => 42)` should be flagged"
        );
    }

    #[test]
    fn test_allows_mock_return_value() {
        let diags = lint("jest.fn().mockReturnValue(42);");
        assert!(diags.is_empty(), "`mockReturnValue` should not be flagged");
    }

    #[test]
    fn test_allows_promise_resolve_implementation() {
        // This is handled by prefer-mock-promise-shorthand, not this rule
        let diags = lint("jest.fn().mockImplementation(() => Promise.resolve(42));");
        assert!(
            diags.is_empty(),
            "Promise.resolve in mockImplementation should not be flagged by this rule"
        );
    }
}
