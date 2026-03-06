//! Rule: `no-negation-in-equality-check`
//!
//! Disallow negation in the left-hand side of equality checks. Expressions
//! like `!a == b` or `!a === b` are parsed as `(!a) == b`, not `a != b`.
//! This is almost always a mistake and leads to confusing behavior.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `!a == b` and `!a === b` patterns where negation binds tighter
/// than the equality operator.
#[derive(Debug)]
pub struct NoNegationInEqualityCheck;

impl NativeRule for NoNegationInEqualityCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-negation-in-equality-check".to_owned(),
            description: "Disallow negation in the left-hand side of equality checks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        // Only check `==` and `===` operators
        if expr.operator != BinaryOperator::Equality
            && expr.operator != BinaryOperator::StrictEquality
        {
            return;
        }

        // Check if the left side is a `!` unary expression
        if let Expression::UnaryExpression(unary) = &expr.left {
            if unary.operator == UnaryOperator::LogicalNot {
                let op_str = if expr.operator == BinaryOperator::Equality {
                    "=="
                } else {
                    "==="
                };
                let negated_op = if expr.operator == BinaryOperator::Equality {
                    "!="
                } else {
                    "!=="
                };

                // Fix: `!a == b` → `a != b`, `!a === b` → `a !== b`
                #[allow(clippy::as_conversions)]
                let fix = {
                    let source = ctx.source_text();
                    let inner_span = unary.argument.span();
                    let right_span = expr.right.span();
                    let inner_text = source
                        .get(inner_span.start as usize..inner_span.end as usize)
                        .unwrap_or("");
                    let right_text = source
                        .get(right_span.start as usize..right_span.end as usize)
                        .unwrap_or("");
                    let replacement = format!("{inner_text} {negated_op} {right_text}");
                    Some(Fix {
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr.span.start, expr.span.end),
                            replacement,
                        }],
                    })
                };

                ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                    rule_name: "no-negation-in-equality-check".to_owned(),
                    message: format!(
                        "Negation in left-hand side of `{op_str}` is confusing — `!a {op_str} b` is parsed as `(!a) {op_str} b`"
                    ),
                    span: Span::new(expr.span.start, expr.span.end),
                    severity: Severity::Warning,
                    help: Some(format!(
                        "Use `a {negated_op} b` instead, or wrap in parentheses: `(!a) {op_str} b`"
                    )),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNegationInEqualityCheck)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_negation_loose_equality() {
        let diags = lint("if (!a == b) {}");
        assert_eq!(diags.len(), 1, "!a == b should be flagged");
    }

    #[test]
    fn test_flags_negation_strict_equality() {
        let diags = lint("if (!a === b) {}");
        assert_eq!(diags.len(), 1, "!a === b should be flagged");
    }

    #[test]
    fn test_allows_inequality() {
        let diags = lint("if (a != b) {}");
        assert!(diags.is_empty(), "a != b should not be flagged");
    }

    #[test]
    fn test_allows_strict_inequality() {
        let diags = lint("if (a !== b) {}");
        assert!(diags.is_empty(), "a !== b should not be flagged");
    }

    #[test]
    fn test_allows_standalone_negation() {
        let diags = lint("if (!a) {}");
        assert!(
            diags.is_empty(),
            "standalone negation should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_equality() {
        let diags = lint("if (a == b) {}");
        assert!(diags.is_empty(), "normal equality should not be flagged");
    }
}
