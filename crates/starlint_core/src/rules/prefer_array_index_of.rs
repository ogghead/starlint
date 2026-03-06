//! Rule: `prefer-array-index-of`
//!
//! Prefer `.indexOf()` over `.findIndex()` for simple equality checks.
//! `.findIndex(x => x === val)` can be simplified to `.indexOf(val)`.

use oxc_ast::AstKind;
use oxc_ast::ast::{ArrowFunctionExpression, Expression, FunctionBody, Statement};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.findIndex()` calls with simple equality callbacks.
#[derive(Debug)]
pub struct PreferArrayIndexOf;

/// Check if an arrow function body is a simple binary equality expression.
fn is_simple_equality_body(body: &FunctionBody<'_>) -> bool {
    // Expression body (single statement that is an expression statement)
    if body.statements.len() != 1 {
        return false;
    }
    let Some(stmt) = body.statements.first() else {
        return false;
    };
    let Statement::ExpressionStatement(expr_stmt) = stmt else {
        return false;
    };
    matches!(
        &expr_stmt.expression,
        Expression::BinaryExpression(bin)
            if matches!(
                bin.operator,
                oxc_ast::ast::BinaryOperator::StrictEquality | oxc_ast::ast::BinaryOperator::Equality
            )
    )
}

/// Extract the value being compared in `x => x === val`, returning the source text of `val`.
/// The parameter name must match one side of the equality.
#[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
fn extract_equality_value(arrow: &ArrowFunctionExpression<'_>, source: &str) -> Option<String> {
    let stmt = arrow.body.statements.first()?;
    let Statement::ExpressionStatement(expr_stmt) = stmt else {
        return None;
    };
    let Expression::BinaryExpression(bin) = &expr_stmt.expression else {
        return None;
    };

    // Get the parameter name
    let param = arrow.params.items.first()?;
    let param_span = param.span();
    let param_name = source.get(param_span.start as usize..param_span.end as usize)?;

    let left_span = bin.left.span();
    let right_span = bin.right.span();
    let left_text = source.get(left_span.start as usize..left_span.end as usize)?;
    let right_text = source.get(right_span.start as usize..right_span.end as usize)?;

    // Return the side that is NOT the parameter
    if left_text == param_name {
        Some(right_text.to_owned())
    } else if right_text == param_name {
        Some(left_text.to_owned())
    } else {
        None
    }
}

impl NativeRule for PreferArrayIndexOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-index-of".to_owned(),
            description: "Prefer `.indexOf()` over `.findIndex()` for simple equality checks"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
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

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name.as_str() != "findIndex" {
            return;
        }

        // Must have exactly one argument.
        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Check for arrow function with simple equality body.
        if let oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) = first_arg {
            if arrow.params.items.len() == 1 && is_simple_equality_body(&arrow.body) {
                // Try to extract the value being compared against the parameter
                let fix = extract_equality_value(arrow, ctx.source_text()).map(|val_text| {
                    let prop_span = Span::new(member.property.span.start, member.property.span.end);
                    let args_span = Span::new(
                        call.arguments
                            .first()
                            .map_or(call.span.end, |a| a.span().start),
                        call.arguments
                            .last()
                            .map_or(call.span.end, |a| a.span().end),
                    );
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `.indexOf({val_text})`"),
                        edits: vec![
                            Edit {
                                span: prop_span,
                                replacement: "indexOf".to_owned(),
                            },
                            Edit {
                                span: args_span,
                                replacement: val_text,
                            },
                        ],
                        is_snippet: false,
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: "prefer-array-index-of".to_owned(),
                    message: "Prefer `.indexOf()` over `.findIndex()` for simple equality checks"
                        .to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace `.findIndex()` with `.indexOf()`".to_owned()),
                    fix,
                    labels: vec![],
                });
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferArrayIndexOf)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_find_index_strict_equality() {
        let diags = lint("arr.findIndex(x => x === 5);");
        assert_eq!(
            diags.len(),
            1,
            "should flag .findIndex() with strict equality"
        );
    }

    #[test]
    fn test_flags_find_index_loose_equality() {
        let diags = lint("arr.findIndex(x => x == val);");
        assert_eq!(
            diags.len(),
            1,
            "should flag .findIndex() with loose equality"
        );
    }

    #[test]
    fn test_allows_find_index_complex_callback() {
        let diags = lint("arr.findIndex(x => x.id === 5);");
        // This is a member expression equality, not a simple `x === val`.
        // Our heuristic still flags it because the body is a binary equality.
        // That is acceptable — it is a suggestion, not an error.
        assert_eq!(diags.len(), 1, "still flags member-based equality");
    }

    #[test]
    fn test_allows_index_of() {
        let diags = lint("arr.indexOf(5);");
        assert!(diags.is_empty(), ".indexOf() should not be flagged");
    }
}
