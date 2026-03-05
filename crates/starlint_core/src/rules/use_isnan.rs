//! Rule: `use-isnan`
//!
//! Require `Number.isNaN()` instead of comparisons with `NaN`.
//! Because `NaN` is unique in JavaScript in that it is not equal to anything,
//! including itself, comparisons like `x === NaN` always evaluate to `false`
//! and `x !== NaN` always evaluates to `true`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags comparisons with `NaN` and suggests using `Number.isNaN()`.
#[derive(Debug)]
pub struct UseIsnan;

impl NativeRule for UseIsnan {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "use-isnan".to_owned(),
            description: "Require `Number.isNaN()` instead of comparisons with `NaN`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        if is_nan(&expr.left) || is_nan(&expr.right) {
            // Determine the non-NaN operand to build the fix
            let non_nan_span = if is_nan(&expr.left) {
                expr.right.span()
            } else {
                expr.left.span()
            };

            #[allow(clippy::as_conversions)]
            let fix = ctx
                .source_text()
                .get(non_nan_span.start as usize..non_nan_span.end as usize)
                .map(|value_text| {
                    // For inequality operators, wrap in negation
                    let is_negated = matches!(
                        expr.operator,
                        oxc_ast::ast::BinaryOperator::StrictInequality
                            | oxc_ast::ast::BinaryOperator::Inequality
                    );
                    let replacement = if is_negated {
                        format!("!Number.isNaN({value_text})")
                    } else {
                        format!("Number.isNaN({value_text})")
                    };
                    Fix {
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr.span.start, expr.span.end),
                            replacement,
                        }],
                    }
                });

            ctx.report(Diagnostic {
                rule_name: "use-isnan".to_owned(),
                message: "Comparisons with `NaN` always produce unexpected results".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: Some("Use `Number.isNaN(value)` instead".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is the identifier `NaN`.
fn is_nan(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(ident) if ident.name == "NaN")
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(UseIsnan)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_strict_equality_nan() {
        let diags = lint("if (x === NaN) {}");
        assert_eq!(diags.len(), 1, "=== NaN should be flagged");
    }

    #[test]
    fn test_flags_loose_equality_nan() {
        let diags = lint("if (x == NaN) {}");
        assert_eq!(diags.len(), 1, "== NaN should be flagged");
    }

    #[test]
    fn test_flags_inequality_nan() {
        let diags = lint("if (x !== NaN) {}");
        assert_eq!(diags.len(), 1, "!== NaN should be flagged");
    }

    #[test]
    fn test_flags_nan_on_left() {
        let diags = lint("if (NaN === x) {}");
        assert_eq!(diags.len(), 1, "NaN on left side should be flagged");
    }

    #[test]
    fn test_flags_less_than_nan() {
        let diags = lint("if (x < NaN) {}");
        assert_eq!(diags.len(), 1, "< NaN should be flagged");
    }

    #[test]
    fn test_allows_number_isnan() {
        let diags = lint("if (Number.isNaN(x)) {}");
        assert!(diags.is_empty(), "Number.isNaN() should not be flagged");
    }

    #[test]
    fn test_allows_isnan() {
        let diags = lint("if (isNaN(x)) {}");
        assert!(diags.is_empty(), "isNaN() should not be flagged");
    }

    #[test]
    fn test_allows_arithmetic_with_nan() {
        let diags = lint("const y = x + NaN;");
        assert!(
            diags.is_empty(),
            "arithmetic with NaN should not be flagged"
        );
    }
}
