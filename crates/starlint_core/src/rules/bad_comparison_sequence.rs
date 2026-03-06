//! Rule: `bad-comparison-sequence` (OXC)
//!
//! Catch chained comparisons like `a < b < c` which don't work as expected in
//! JavaScript. In `a < b < c`, `a < b` evaluates to a boolean, which is then
//! compared to `c` — almost never the intended behavior.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags chained comparison sequences like `a < b < c`.
#[derive(Debug)]
pub struct BadComparisonSequence;

impl NativeRule for BadComparisonSequence {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-comparison-sequence".to_owned(),
            description: "Catch chained comparison sequences like `a < b < c`".to_owned(),
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

        // Only check comparison operators (not equality)
        if !expr.operator.is_compare() {
            return;
        }

        // Check if the left operand is also a comparison — that makes it a chain
        if let Expression::BinaryExpression(left) = &expr.left {
            if left.operator.is_compare() {
                // Fix: a < b < c → a < b && b < c
                #[allow(clippy::as_conversions)]
                let fix = {
                    let source = ctx.source_text();
                    let a_span = left.left.span();
                    let b_span = left.right.span();
                    let c_span = expr.right.span();
                    let a = source.get(a_span.start as usize..a_span.end as usize);
                    let b = source.get(b_span.start as usize..b_span.end as usize);
                    let c = source.get(c_span.start as usize..c_span.end as usize);
                    match (a, b, c) {
                        (Some(a_text), Some(b_text), Some(c_text)) => {
                            let left_op = operator_str(left.operator);
                            let right_op = operator_str(expr.operator);
                            let replacement = format!(
                                "{a_text} {left_op} {b_text} && {b_text} {right_op} {c_text}"
                            );
                            Some(Fix {
                                message: format!("Replace with `{replacement}`"),
                                edits: vec![Edit {
                                    span: Span::new(expr.span.start, expr.span.end),
                                    replacement,
                                }],
                                is_snippet: false,
                            })
                        }
                        _ => None,
                    }
                };

                ctx.report(Diagnostic {
                    rule_name: "bad-comparison-sequence".to_owned(),
                    message: "Chained comparisons like `a < b < c` do not work as expected in JavaScript — \
                     the left comparison returns a boolean, which is then compared to the right operand".to_owned(),
                    span: Span::new(expr.span.start, expr.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

/// Convert a comparison operator to its source string.
const fn operator_str(op: BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::LessThan => "<",
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::LessEqualThan => "<=",
        BinaryOperator::GreaterEqualThan => ">=",
        _ => "??",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BadComparisonSequence)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_chained_less_than() {
        let diags = lint("if (a < b < c) {}");
        assert_eq!(diags.len(), 1, "a < b < c should be flagged");
    }

    #[test]
    fn test_flags_chained_greater_than() {
        let diags = lint("if (a > b > c) {}");
        assert_eq!(diags.len(), 1, "a > b > c should be flagged");
    }

    #[test]
    fn test_flags_mixed_chain() {
        let diags = lint("if (a < b >= c) {}");
        assert_eq!(diags.len(), 1, "a < b >= c should be flagged");
    }

    #[test]
    fn test_allows_simple_comparison() {
        let diags = lint("if (a < b) {}");
        assert!(diags.is_empty(), "simple comparison should not be flagged");
    }

    #[test]
    fn test_allows_logical_and_comparisons() {
        let diags = lint("if (a < b && b < c) {}");
        assert!(
            diags.is_empty(),
            "proper range check with && should not be flagged"
        );
    }
}
