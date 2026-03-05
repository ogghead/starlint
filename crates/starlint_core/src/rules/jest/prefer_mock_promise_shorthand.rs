//! Rule: `jest/prefer-mock-promise-shorthand`
//!
//! Suggest `jest.fn().mockResolvedValue(x)` over
//! `jest.fn().mockImplementation(() => Promise.resolve(x))`. The shorthand
//! methods are more readable and clearly express intent.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.mockImplementation(() => Promise.resolve(x))` patterns.
#[derive(Debug)]
pub struct PreferMockPromiseShorthand;

impl NativeRule for PreferMockPromiseShorthand {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-mock-promise-shorthand".to_owned(),
            description: "Suggest using `mockResolvedValue`/`mockRejectedValue` shorthand"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

        // Must be `.mockImplementation(...)` or `.mockReturnValue(...)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let method = member.property.name.as_str();
        if method != "mockImplementation" && method != "mockReturnValue" {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };
        let Some(arg_expr) = first_arg.as_expression() else {
            return;
        };

        // For mockImplementation: check if the argument is an arrow/function
        // that returns Promise.resolve or Promise.reject
        if method == "mockImplementation" {
            let return_expr = match arg_expr {
                Expression::ArrowFunctionExpression(arrow) => {
                    // Check for expression body: `() => Promise.resolve(x)`
                    if arrow.expression {
                        get_single_expression_body(&arrow.body)
                    } else {
                        // Check for `() => { return Promise.resolve(x); }`
                        get_single_return_expression(&arrow.body)
                    }
                }
                Expression::FunctionExpression(func) => func
                    .body
                    .as_ref()
                    .and_then(|b| get_single_return_expression(b)),
                _ => None,
            };

            let Some(ret) = return_expr else {
                return;
            };
            if let Some(promise_method) = is_promise_call(ret) {
                let suggestion = match promise_method {
                    "resolve" => "mockResolvedValue",
                    "reject" => "mockRejectedValue",
                    _ => return,
                };
                let fix =
                    build_mock_shorthand_fix(call, member, suggestion, ret, ctx.source_text());
                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-mock-promise-shorthand".to_owned(),
                    message: format!(
                        "Use `.{suggestion}()` instead of `.mockImplementation(() => Promise.{promise_method}(...))`"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace with `.{suggestion}()`")),
                    fix,
                    labels: vec![],
                });
            }
        } else if method == "mockReturnValue" {
            // Check if the argument is `Promise.resolve(x)` or `Promise.reject(x)`
            if let Some(promise_method) = is_promise_call(arg_expr) {
                let suggestion = match promise_method {
                    "resolve" => "mockResolvedValue",
                    "reject" => "mockRejectedValue",
                    _ => return,
                };
                let fix =
                    build_mock_shorthand_fix(call, member, suggestion, arg_expr, ctx.source_text());
                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-mock-promise-shorthand".to_owned(),
                    message: format!(
                        "Use `.{suggestion}()` instead of `.mockReturnValue(Promise.{promise_method}(...))`"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace with `.{suggestion}()`")),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

/// Build fix: replace `.mockImplementation(() => Promise.resolve(x))` with `.mockResolvedValue(x)`.
#[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
fn build_mock_shorthand_fix(
    call: &oxc_ast::ast::CallExpression<'_>,
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    suggestion: &str,
    promise_expr: &Expression<'_>,
    source: &str,
) -> Option<Fix> {
    // Extract the object before `.mockImplementation(...)` / `.mockReturnValue(...)`
    let obj_span = member.object.span();
    let obj_text = source.get(obj_span.start as usize..obj_span.end as usize)?;

    // Extract the argument from Promise.resolve(x) / Promise.reject(x)
    let Expression::CallExpression(promise_call) = promise_expr else {
        return None;
    };
    let inner_arg_text = promise_call.arguments.first().map(|a| {
        let sp = a.span();
        source.get(sp.start as usize..sp.end as usize).unwrap_or("")
    });
    let arg_text = inner_arg_text.unwrap_or("");

    let replacement = format!("{obj_text}.{suggestion}({arg_text})");
    Some(Fix {
        message: format!("Replace with `.{suggestion}()`"),
        edits: vec![Edit {
            span: Span::new(call.span.start, call.span.end),
            replacement,
        }],
    })
}

/// Check if an expression is `Promise.resolve(...)` or `Promise.reject(...)`.
/// Returns the method name ("resolve" or "reject") if matched.
fn is_promise_call<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    let Expression::Identifier(obj) = &member.object else {
        return None;
    };
    if obj.name.as_str() != "Promise" {
        return None;
    }
    let method = member.property.name.as_str();
    (method == "resolve" || method == "reject").then_some(method)
}

/// Get the single expression from an arrow function expression body.
fn get_single_expression_body<'a>(
    body: &'a oxc_ast::ast::FunctionBody<'a>,
) -> Option<&'a Expression<'a>> {
    // For expression arrows, the body has a single ExpressionStatement
    body.statements.first().and_then(|stmt| {
        if let oxc_ast::ast::Statement::ExpressionStatement(es) = stmt {
            Some(&es.expression)
        } else {
            None
        }
    })
}

/// Get the expression from a function body with a single return statement.
fn get_single_return_expression<'a>(
    body: &'a oxc_ast::ast::FunctionBody<'a>,
) -> Option<&'a Expression<'a>> {
    if body.statements.len() != 1 {
        return None;
    }
    body.statements.first().and_then(|stmt| {
        if let oxc_ast::ast::Statement::ReturnStatement(ret) = stmt {
            ret.argument.as_ref()
        } else {
            None
        }
    })
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferMockPromiseShorthand)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_mock_implementation_promise_resolve() {
        let diags = lint("jest.fn().mockImplementation(() => Promise.resolve(42));");
        assert_eq!(
            diags.len(),
            1,
            "mockImplementation with Promise.resolve should be flagged"
        );
    }

    #[test]
    fn test_flags_mock_return_value_promise_reject() {
        let diags = lint("jest.fn().mockReturnValue(Promise.reject(new Error('fail')));");
        assert_eq!(
            diags.len(),
            1,
            "mockReturnValue with Promise.reject should be flagged"
        );
    }

    #[test]
    fn test_allows_mock_resolved_value() {
        let diags = lint("jest.fn().mockResolvedValue(42);");
        assert!(
            diags.is_empty(),
            "`mockResolvedValue` should not be flagged"
        );
    }
}
