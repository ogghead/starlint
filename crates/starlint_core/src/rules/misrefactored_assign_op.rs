//! Rule: `misrefactored-assign-op` (OXC)
//!
//! Detect misrefactored assignment operators like `a -= a - b` which was
//! probably meant to be `a -= b` (or `a = a - b`). Similarly `a += a + b`
//! probably meant `a += b`.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentOperator, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags misrefactored compound assignment operators.
#[derive(Debug)]
pub struct MisrefactoredAssignOp;

impl NativeRule for MisrefactoredAssignOp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "misrefactored-assign-op".to_owned(),
            description: "Detect `a -= a - b` (probably meant `a -= b`)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Only check compound assignment operators
        let corresponding_binary = match assign.operator {
            AssignmentOperator::Addition => Some(oxc_ast::ast::BinaryOperator::Addition),
            AssignmentOperator::Subtraction => Some(oxc_ast::ast::BinaryOperator::Subtraction),
            AssignmentOperator::Multiplication => {
                Some(oxc_ast::ast::BinaryOperator::Multiplication)
            }
            AssignmentOperator::Division => Some(oxc_ast::ast::BinaryOperator::Division),
            AssignmentOperator::Remainder => Some(oxc_ast::ast::BinaryOperator::Remainder),
            _ => None,
        };

        let Some(expected_op) = corresponding_binary else {
            return;
        };

        // The RHS should be a binary expression with the same operator
        let Expression::BinaryExpression(rhs_bin) = &assign.right else {
            return;
        };

        if rhs_bin.operator != expected_op {
            return;
        }

        // Check if the left side of the binary expression matches the assignment target
        // e.g., `a -= a - b` → the `a` in `a - b` matches the `a` being assigned
        let target_span = assignment_target_span(&assign.left);
        let lhs_span = expression_span(&rhs_bin.left);

        if let (Some(target), Some(lhs)) = (target_span, lhs_span) {
            // Compare by source text length (same identifier = same span length at same content)
            let source = ctx.source_text();
            let target_src = &source[target.0..target.1];
            let lhs_src = &source[lhs.0..lhs.1];

            if target_src == lhs_src {
                let op_str = match assign.operator {
                    AssignmentOperator::Addition => "+=",
                    AssignmentOperator::Subtraction => "-=",
                    AssignmentOperator::Multiplication => "*=",
                    AssignmentOperator::Division => "/=",
                    AssignmentOperator::Remainder => "%=",
                    _ => return,
                };
                let findings = vec![(
                    format!(
                        "`{target_src} {op_str} {target_src} ...` looks like a misrefactored \
                         compound assignment — did you mean `{target_src} {op_str} ...` without \
                         repeating `{target_src}`?"
                    ),
                    Span::new(assign.span.start, assign.span.end),
                )];
                for (msg, span) in findings {
                    ctx.report_warning("misrefactored-assign-op", &msg, span);
                }
            }
        }
    }
}

/// Get the span (as usize range) of an assignment target.
fn assignment_target_span(target: &oxc_ast::ast::AssignmentTarget<'_>) -> Option<(usize, usize)> {
    match target {
        oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) => {
            usize::try_from(id.span.start)
                .ok()
                .zip(usize::try_from(id.span.end).ok())
        }
        oxc_ast::ast::AssignmentTarget::StaticMemberExpression(m) => usize::try_from(m.span.start)
            .ok()
            .zip(usize::try_from(m.span.end).ok()),
        oxc_ast::ast::AssignmentTarget::ComputedMemberExpression(m) => {
            usize::try_from(m.span.start)
                .ok()
                .zip(usize::try_from(m.span.end).ok())
        }
        _ => None,
    }
}

/// Get the span (as usize range) of an expression.
fn expression_span(expr: &Expression<'_>) -> Option<(usize, usize)> {
    match expr {
        Expression::Identifier(id) => usize::try_from(id.span.start)
            .ok()
            .zip(usize::try_from(id.span.end).ok()),
        Expression::StaticMemberExpression(m) => usize::try_from(m.span.start)
            .ok()
            .zip(usize::try_from(m.span.end).ok()),
        Expression::ComputedMemberExpression(m) => usize::try_from(m.span.start)
            .ok()
            .zip(usize::try_from(m.span.end).ok()),
        _ => None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MisrefactoredAssignOp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_subtract_assign() {
        let diags = lint("a -= a - b;");
        assert_eq!(diags.len(), 1, "a -= a - b should be flagged");
    }

    #[test]
    fn test_flags_add_assign() {
        let diags = lint("a += a + b;");
        assert_eq!(diags.len(), 1, "a += a + b should be flagged");
    }

    #[test]
    fn test_allows_correct_assign() {
        let diags = lint("a -= b;");
        assert!(
            diags.is_empty(),
            "correct compound assignment should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_target() {
        let diags = lint("a -= b - c;");
        assert!(
            diags.is_empty(),
            "different target in RHS should not be flagged"
        );
    }

    #[test]
    fn test_allows_simple_assignment() {
        let diags = lint("a = a - b;");
        assert!(diags.is_empty(), "simple assignment should not be flagged");
    }
}
