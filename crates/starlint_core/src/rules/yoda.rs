//! Rule: `yoda`
//!
//! Disallow "Yoda conditions" where the literal comes before the variable
//! in a comparison, e.g. `"red" === color` instead of `color === "red"`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags Yoda conditions (literal on the left of a comparison).
#[derive(Debug)]
pub struct Yoda;

impl NativeRule for Yoda {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "yoda".to_owned(),
            description: "Disallow Yoda conditions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        // Flag: literal on the left, non-literal on the right
        if is_literal(&expr.left) && !is_literal(&expr.right) {
            let source = ctx.source_text();
            let left_start = usize::try_from(expr.left.span().start).unwrap_or(0);
            let left_end = usize::try_from(expr.left.span().end).unwrap_or(0);
            let right_start = usize::try_from(expr.right.span().start).unwrap_or(0);
            let right_end = usize::try_from(expr.right.span().end).unwrap_or(0);
            let left_text = source.get(left_start..left_end).unwrap_or("");
            let right_text = source.get(right_start..right_end).unwrap_or("");
            let flipped = flip_comparison(expr.operator);

            ctx.report(Diagnostic {
                rule_name: "yoda".to_owned(),
                message: "Expected literal to be on the right side of comparison".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some("Swap the operands".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Swap operands".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: format!("{right_text} {flipped} {left_text}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Flip a comparison operator for swapping operands.
const fn flip_comparison(op: BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::LessThan => ">",
        BinaryOperator::LessEqualThan => ">=",
        BinaryOperator::GreaterThan => "<",
        BinaryOperator::GreaterEqualThan => "<=",
        BinaryOperator::Equality => "==",
        BinaryOperator::Inequality => "!=",
        BinaryOperator::StrictInequality => "!==",
        _ => "===",
    }
}

/// Check if an expression is a literal value.
const fn is_literal(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
    )
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Yoda)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_yoda_condition() {
        let diags = lint("if ('red' === color) {}");
        assert_eq!(diags.len(), 1, "Yoda condition should be flagged");
    }

    #[test]
    fn test_allows_normal_condition() {
        let diags = lint("if (color === 'red') {}");
        assert!(diags.is_empty(), "normal condition should not be flagged");
    }

    #[test]
    fn test_allows_two_variables() {
        let diags = lint("if (a === b) {}");
        assert!(diags.is_empty(), "two variables should not be flagged");
    }

    #[test]
    fn test_flags_number_yoda() {
        let diags = lint("if (5 < x) {}");
        assert_eq!(diags.len(), 1, "number yoda condition should be flagged");
    }
}
