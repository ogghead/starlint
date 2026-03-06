//! Rule: `no-self-compare`
//!
//! Disallow comparisons where both sides are exactly the same.
//! Comparing a value against itself is almost always a bug. The only
//! valid use case (`x !== x` to check for `NaN`) should use `Number.isNaN()`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags comparisons where both operands are the same identifier.
#[derive(Debug)]
pub struct NoSelfCompare;

impl NativeRule for NoSelfCompare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-self-compare".to_owned(),
            description: "Disallow comparisons where both sides are the same".to_owned(),
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

        // Compare source text of both sides to detect identical expressions.
        let left_span = expr.left.span();
        let right_span = expr.right.span();

        let left_start = usize::try_from(left_span.start).unwrap_or(0);
        let left_end = usize::try_from(left_span.end).unwrap_or(0);
        let right_start = usize::try_from(right_span.start).unwrap_or(0);
        let right_end = usize::try_from(right_span.end).unwrap_or(0);

        let source = ctx.source_text();
        let left_text = source.get(left_start..left_end);
        let right_text = source.get(right_start..right_end);

        if let (Some(left), Some(right)) = (left_text, right_text) {
            if !left.is_empty() && left == right {
                // For `x !== x`, offer fix to `Number.isNaN(x)`
                let fix = matches!(
                    expr.operator,
                    oxc_ast::ast::BinaryOperator::StrictInequality
                )
                .then(|| {
                    let replacement = format!("Number.isNaN({left})");
                    Fix {
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr.span.start, expr.span.end),
                            replacement,
                        }],
                    }
                });

                ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                    rule_name: "no-self-compare".to_owned(),
                    message: format!("Comparing `{left}` against itself is always predictable"),
                    span: Span::new(expr.span.start, expr.span.end),
                    severity: Severity::Error,
                    help: Some("If testing for NaN, use `Number.isNaN(value)` instead".to_owned()),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSelfCompare)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_self_strict_equality() {
        let diags = lint("if (x === x) {}");
        assert_eq!(diags.len(), 1, "x === x should be flagged");
    }

    #[test]
    fn test_flags_self_inequality() {
        let diags = lint("if (x !== x) {}");
        assert_eq!(
            diags.len(),
            1,
            "x !== x should be flagged (use Number.isNaN)"
        );
    }

    #[test]
    fn test_flags_self_less_than() {
        let diags = lint("if (x < x) {}");
        assert_eq!(diags.len(), 1, "x < x should be flagged");
    }

    #[test]
    fn test_allows_different_operands() {
        let diags = lint("if (x === y) {}");
        assert!(diags.is_empty(), "different operands should not be flagged");
    }

    #[test]
    fn test_allows_arithmetic() {
        let diags = lint("const y = x + x;");
        assert!(diags.is_empty(), "arithmetic is not a comparison");
    }
}
