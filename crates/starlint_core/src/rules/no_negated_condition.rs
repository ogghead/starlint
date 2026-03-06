//! Rule: `no-negated-condition`
//!
//! Disallow negated conditions in `if` statements with an `else` branch
//! and in ternary operators. These are harder to read and should be
//! inverted for clarity.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags negated conditions that should be inverted.
#[derive(Debug)]
pub struct NoNegatedCondition;

impl NativeRule for NoNegatedCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-negated-condition".to_owned(),
            description: "Disallow negated conditions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ConditionalExpression, AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::IfStatement(stmt) => {
                // Only flag if there is an else branch
                if stmt.alternate.is_none() {
                    return;
                }
                if is_negated(&stmt.test) {
                    // if-else autofix is too complex (multi-line blocks), just report
                    ctx.report_warning(
                        "no-negated-condition",
                        "Unexpected negated condition in `if` with `else`",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstKind::ConditionalExpression(expr) => {
                if is_negated(&expr.test) {
                    let source = ctx.source_text();
                    let negated_text = negate_condition(&expr.test, source);
                    let cons_text = span_text(&expr.consequent, source).to_owned();
                    let alt_text = span_text(&expr.alternate, source).to_owned();

                    // `!x ? a : b` → `x ? b : a`
                    let replacement = format!("{negated_text} ? {alt_text} : {cons_text}");

                    ctx.report(Diagnostic {
                        rule_name: "no-negated-condition".to_owned(),
                        message: "Unexpected negated condition in ternary".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Invert the condition and swap branches".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Invert condition and swap branches".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Get source text for an expression span.
fn span_text<'s>(expr: &Expression<'_>, source: &'s str) -> &'s str {
    let sp = expr.span();
    let start = usize::try_from(sp.start).unwrap_or(0);
    let end = usize::try_from(sp.end).unwrap_or(start);
    source.get(start..end).unwrap_or_default()
}

/// Produce the negated form of a condition.
/// `!x` → `x`, `a !== b` → `a === b`, `a != b` → `a == b`.
fn negate_condition(expr: &Expression<'_>, source: &str) -> String {
    match expr {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            span_text(&unary.argument, source).to_owned()
        }
        Expression::BinaryExpression(binary) => {
            let left = span_text(&binary.left, source);
            let right = span_text(&binary.right, source);
            let new_op = match binary.operator {
                oxc_ast::ast::BinaryOperator::StrictInequality => "===",
                oxc_ast::ast::BinaryOperator::Inequality => "==",
                _ => return span_text(expr, source).to_owned(),
            };
            format!("{left} {new_op} {right}")
        }
        _ => span_text(expr, source).to_owned(),
    }
}

/// Check if an expression is a negation (`!x`) or inequality (`a !== b`, `a != b`).
fn is_negated(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary) => {
            matches!(
                binary.operator,
                oxc_ast::ast::BinaryOperator::StrictInequality
                    | oxc_ast::ast::BinaryOperator::Inequality
            )
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNegatedCondition)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_negated_if_with_else() {
        let diags = lint("if (!x) { a(); } else { b(); }");
        assert_eq!(diags.len(), 1, "negated if with else should be flagged");
    }

    #[test]
    fn test_allows_negated_if_without_else() {
        let diags = lint("if (!x) { a(); }");
        assert!(
            diags.is_empty(),
            "negated if without else should not be flagged"
        );
    }

    #[test]
    fn test_flags_negated_ternary() {
        let diags = lint("var r = !x ? a : b;");
        assert_eq!(diags.len(), 1, "negated ternary should be flagged");
    }

    #[test]
    fn test_allows_non_negated_ternary() {
        let diags = lint("var r = x ? a : b;");
        assert!(
            diags.is_empty(),
            "non-negated ternary should not be flagged"
        );
    }

    #[test]
    fn test_flags_inequality_if_with_else() {
        let diags = lint("if (a !== b) { x(); } else { y(); }");
        assert_eq!(diags.len(), 1, "inequality if with else should be flagged");
    }
}
