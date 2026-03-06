//! Rule: `double-comparisons` (OXC)
//!
//! Detect `a >= b && a <= b` which can be simplified to `a === b`, and
//! `a > b || a < b` which can be simplified to `a !== b`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, LogicalOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags redundant double comparisons that can be simplified.
#[derive(Debug)]
pub struct DoubleComparisons;

impl NativeRule for DoubleComparisons {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "double-comparisons".to_owned(),
            description: "Detect `a >= b && a <= b` (simplify to `a === b`)".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::LogicalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::LogicalExpression(logical) = kind else {
            return;
        };

        let Expression::BinaryExpression(left) = &logical.left else {
            return;
        };
        let Expression::BinaryExpression(right) = &logical.right else {
            return;
        };

        // Extract source text for operand comparison
        let source = ctx.source_text();
        let same_operands = (src_equal(&left.left, &right.left, source)
            && src_equal(&left.right, &right.right, source))
            || (src_equal(&left.left, &right.right, source)
                && src_equal(&left.right, &right.left, source));

        if !same_operands {
            return;
        }

        let finding = match logical.operator {
            LogicalOperator::And => {
                let pair = (left.operator, right.operator);
                matches!(
                    pair,
                    (
                        BinaryOperator::GreaterEqualThan,
                        BinaryOperator::LessEqualThan
                    ) | (
                        BinaryOperator::LessEqualThan,
                        BinaryOperator::GreaterEqualThan
                    )
                )
                .then_some("This double comparison can be simplified to `===`")
            }
            LogicalOperator::Or => {
                let pair = (left.operator, right.operator);
                matches!(
                    pair,
                    (BinaryOperator::GreaterThan, BinaryOperator::LessThan)
                        | (BinaryOperator::LessThan, BinaryOperator::GreaterThan)
                )
                .then_some("This double comparison can be simplified to `!==`")
            }
            LogicalOperator::Coalesce => None,
        };

        if let Some(message) = finding {
            let replacement_op = match logical.operator {
                LogicalOperator::And => "===",
                _ => "!==",
            };
            let left_a = expr_source(&left.left, source).unwrap_or("");
            let right_b = expr_source(&left.right, source).unwrap_or("");

            ctx.report(Diagnostic {
                rule_name: "double-comparisons".to_owned(),
                message: message.to_owned(),
                span: Span::new(logical.span.start, logical.span.end),
                severity: Severity::Warning,
                help: Some(format!("Simplify to `{replacement_op}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Simplify to `{replacement_op}`"),
                    edits: vec![Edit {
                        span: Span::new(logical.span.start, logical.span.end),
                        replacement: format!("{left_a} {replacement_op} {right_b}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Compare two expressions by their source text content.
fn src_equal(a: &Expression<'_>, b: &Expression<'_>, source: &str) -> bool {
    let a_slice = expr_source(a, source);
    let b_slice = expr_source(b, source);
    match (a_slice, b_slice) {
        (Some(a_str), Some(b_str)) => a_str == b_str,
        _ => false,
    }
}

/// Extract the source text slice for an expression.
fn expr_source<'s>(expr: &Expression<'_>, source: &'s str) -> Option<&'s str> {
    let start = usize::try_from(expr.span().start).ok()?;
    let end = usize::try_from(expr.span().end).ok()?;
    source.get(start..end)
}

use oxc_span::GetSpan;

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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DoubleComparisons)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_gte_and_lte() {
        let diags = lint("if (a >= b && a <= b) {}");
        assert_eq!(
            diags.len(),
            1,
            "a >= b && a <= b should be flagged (simplify to ===)"
        );
    }

    #[test]
    fn test_flags_gt_or_lt() {
        let diags = lint("if (a > b || a < b) {}");
        assert_eq!(
            diags.len(),
            1,
            "a > b || a < b should be flagged (simplify to !==)"
        );
    }

    #[test]
    fn test_allows_different_operands() {
        let diags = lint("if (a >= b && c <= d) {}");
        assert!(diags.is_empty(), "different operands should not be flagged");
    }

    #[test]
    fn test_allows_normal_range_check() {
        let diags = lint("if (a >= 0 && a <= 10) {}");
        assert!(
            diags.is_empty(),
            "range check with different bounds should not be flagged"
        );
    }
}
