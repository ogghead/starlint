//! Rule: `operator-assignment`
//!
//! Require or disallow assignment operator shorthand where possible.
//! `x = x + 1` should be written as `x += 1`.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, BinaryOperator, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags assignments that could use shorthand operators.
#[derive(Debug)]
pub struct OperatorAssignment;

impl NativeRule for OperatorAssignment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "operator-assignment".to_owned(),
            description: "Require assignment operator shorthand where possible".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Only check plain assignment (=)
        if assign.operator != AssignmentOperator::Assign {
            return;
        }

        // Right side must be a binary expression
        let Expression::BinaryExpression(binary) = &assign.right else {
            return;
        };

        // Check if the binary operator has a compound assignment form
        if !has_shorthand(binary.operator) {
            return;
        }

        // Check if the left side of the binary matches the assignment target
        if targets_match(&assign.left, &binary.left, ctx.source_text()) {
            ctx.report_warning(
                "operator-assignment",
                "Assignment can be replaced with an operator assignment",
                Span::new(assign.span.start, assign.span.end),
            );
        }
    }
}

/// Check if a binary operator has a corresponding compound assignment operator.
const fn has_shorthand(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
    )
}

/// Compare assignment target and expression by source text.
fn targets_match(target: &AssignmentTarget<'_>, expr: &Expression<'_>, source: &str) -> bool {
    use oxc_span::GetSpan;

    let target_span = target.span();
    let expr_span = expr.span();

    let target_start = usize::try_from(target_span.start).unwrap_or(0);
    let target_end = usize::try_from(target_span.end).unwrap_or(0);
    let expr_start = usize::try_from(expr_span.start).unwrap_or(0);
    let expr_end = usize::try_from(expr_span.end).unwrap_or(0);

    let target_text = source.get(target_start..target_end);
    let expr_text = source.get(expr_start..expr_end);

    match (target_text, expr_text) {
        (Some(t), Some(e)) => t == e,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(OperatorAssignment)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_add_assignment() {
        let diags = lint("x = x + 1;");
        assert_eq!(
            diags.len(),
            1,
            "x = x + 1 should be flagged"
        );
    }

    #[test]
    fn test_allows_shorthand() {
        let diags = lint("x += 1;");
        assert!(
            diags.is_empty(),
            "x += 1 should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_variables() {
        let diags = lint("x = y + 1;");
        assert!(
            diags.is_empty(),
            "x = y + 1 should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiply_assignment() {
        let diags = lint("x = x * 2;");
        assert_eq!(
            diags.len(),
            1,
            "x = x * 2 should be flagged"
        );
    }
}
