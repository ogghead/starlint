//! Rule: `for-direction`
//!
//! Enforce `for` loop update clause moving the counter in the right direction.
//! A `for` loop with a stop condition that can never be reached (e.g.
//! `for (i = 0; i < 10; i--)`) is almost certainly a bug.

use oxc_ast::AstKind;
use oxc_ast::ast::{
    AssignmentOperator, BinaryOperator, Expression, SimpleAssignmentTarget, UpdateOperator,
};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `for` loops whose update clause moves the counter away from the stop
/// condition, making the loop either infinite or immediately terminating.
#[derive(Debug)]
pub struct ForDirection;

impl NativeRule for ForDirection {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "for-direction".to_owned(),
            description:
                "Enforce `for` loop update clause moving the counter toward the stop condition"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ForStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ForStatement(stmt) = kind else {
            return;
        };

        // We need both a test (stop condition) and an update clause.
        let (Some(test), Some(update)) = (&stmt.test, &stmt.update) else {
            return;
        };

        // Extract the comparison operator and the counter name from the test.
        let Some((counter_name, direction)) = extract_test_direction(test) else {
            return;
        };

        // Check whether the update moves the counter in the wrong direction.
        if moves_wrong_direction(update, counter_name, direction) {
            // Fix: swap the update direction
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let update_span = update.span();
                source
                    .get(update_span.start as usize..update_span.end as usize)
                    .and_then(|update_text| {
                        let replacement = swap_update_direction(update_text)?;
                        Some(Fix {
                            message: format!("Replace `{update_text}` with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(update_span.start, update_span.end),
                                replacement,
                            }],
                        })
                    })
            };

            ctx.report(Diagnostic {
                rule_name: "for-direction".to_owned(),
                message: format!(
                    "The update clause in this `for` loop moves the counter `{counter_name}` in the wrong direction"
                ),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Error,
                help: Some(
                    "The loop counter should move toward the stop condition, not away from it"
                        .to_owned(),
                ),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Which direction the counter must move based on the comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CounterDirection {
    /// Counter must increase (e.g. `i < 10`).
    Increasing,
    /// Counter must decrease (e.g. `i > 0`).
    Decreasing,
}

/// Given a test expression like `i < 10`, extract the counter variable name
/// and the required direction. Returns `None` if the test is not a simple
/// comparison with an identifier on one side.
fn extract_test_direction<'a>(test: &'a Expression<'a>) -> Option<(&'a str, CounterDirection)> {
    let Expression::BinaryExpression(bin) = test else {
        return None;
    };

    match bin.operator {
        // `counter < x` or `counter <= x` → counter must increase
        BinaryOperator::LessThan | BinaryOperator::LessEqualThan => {
            identifier_name(&bin.left).map(|name| (name, CounterDirection::Increasing))
        }
        // `counter > x` or `counter >= x` → counter must decrease
        BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan => {
            identifier_name(&bin.left).map(|name| (name, CounterDirection::Decreasing))
        }
        _ => None,
    }
}

/// Extract a plain identifier name from an expression.
fn identifier_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    if let Expression::Identifier(ident) = expr {
        Some(ident.name.as_str())
    } else {
        None
    }
}

/// Check if the update expression moves the named counter in the wrong direction.
fn moves_wrong_direction(
    update: &Expression<'_>,
    counter_name: &str,
    required: CounterDirection,
) -> bool {
    match update {
        // `i++` or `i--`
        Expression::UpdateExpression(upd) => {
            let target_name = simple_assignment_target_name(&upd.argument);
            if target_name != Some(counter_name) {
                return false;
            }
            match upd.operator {
                UpdateOperator::Increment => required == CounterDirection::Decreasing,
                UpdateOperator::Decrement => required == CounterDirection::Increasing,
            }
        }
        // `i += 1` or `i -= 1`
        Expression::AssignmentExpression(assign) => {
            let target_name = assignment_target_name(&assign.left);
            if target_name != Some(counter_name) {
                return false;
            }
            match assign.operator {
                AssignmentOperator::Addition => {
                    is_positive_numeric(&assign.right) && required == CounterDirection::Decreasing
                }
                AssignmentOperator::Subtraction => {
                    is_positive_numeric(&assign.right) && required == CounterDirection::Increasing
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Get the name from a `SimpleAssignmentTarget` if it's a plain identifier.
fn simple_assignment_target_name<'a>(target: &'a SimpleAssignmentTarget<'a>) -> Option<&'a str> {
    if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
        Some(ident.name.as_str())
    } else {
        None
    }
}

/// Get the name from an `AssignmentTarget` if it's a plain identifier.
fn assignment_target_name<'a>(target: &'a oxc_ast::ast::AssignmentTarget<'a>) -> Option<&'a str> {
    match target {
        oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            Some(ident.name.as_str())
        }
        _ => None,
    }
}

/// Check if an expression is a positive numeric literal (not zero).
fn is_positive_numeric(expr: &Expression<'_>) -> bool {
    if let Expression::NumericLiteral(lit) = expr {
        lit.value > 0.0
    } else {
        false
    }
}

/// Swap the direction of an update expression in source text.
/// `i++` → `i--`, `i--` → `i++`, `i += 1` → `i -= 1`, `i -= 1` → `i += 1`
fn swap_update_direction(text: &str) -> Option<String> {
    if text.ends_with("++") {
        Some(format!("{}--", &text[..text.len().saturating_sub(2)]))
    } else if text.ends_with("--") {
        Some(format!("{}++", &text[..text.len().saturating_sub(2)]))
    } else if let Some(rest) = text.strip_prefix("++") {
        Some(format!("--{rest}"))
    } else if let Some(rest) = text.strip_prefix("--") {
        Some(format!("++{rest}"))
    } else if let Some(pos) = text.find("+=") {
        let mut result = String::with_capacity(text.len());
        result.push_str(&text[..pos]);
        result.push_str("-=");
        result.push_str(&text[pos.saturating_add(2)..]);
        Some(result)
    } else if let Some(pos) = text.find("-=") {
        let mut result = String::with_capacity(text.len());
        result.push_str(&text[..pos]);
        result.push_str("+=");
        result.push_str(&text[pos.saturating_add(2)..]);
        Some(result)
    } else {
        None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ForDirection)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_correct_increment() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert!(diags.is_empty(), "correct for loop should not be flagged");
    }

    #[test]
    fn test_correct_decrement() {
        let diags = lint("for (let i = 10; i > 0; i--) {}");
        assert!(
            diags.is_empty(),
            "correct decrement loop should not be flagged"
        );
    }

    #[test]
    fn test_wrong_direction_increment() {
        let diags = lint("for (let i = 0; i < 10; i--) {}");
        assert_eq!(diags.len(), 1, "decrementing toward < should be flagged");
    }

    #[test]
    fn test_wrong_direction_decrement() {
        let diags = lint("for (let i = 10; i > 0; i++) {}");
        assert_eq!(diags.len(), 1, "incrementing toward > should be flagged");
    }

    #[test]
    fn test_wrong_direction_assignment_add() {
        let diags = lint("for (let i = 10; i > 0; i += 1) {}");
        assert_eq!(diags.len(), 1, "adding toward > should be flagged");
    }

    #[test]
    fn test_wrong_direction_assignment_sub() {
        let diags = lint("for (let i = 0; i < 10; i -= 1) {}");
        assert_eq!(diags.len(), 1, "subtracting toward < should be flagged");
    }

    #[test]
    fn test_correct_assignment_add() {
        let diags = lint("for (let i = 0; i < 10; i += 1) {}");
        assert!(diags.is_empty(), "adding toward < should not be flagged");
    }

    #[test]
    fn test_no_test_clause() {
        let diags = lint("for (let i = 0; ; i++) {}");
        assert!(diags.is_empty(), "no test clause should not be flagged");
    }

    #[test]
    fn test_no_update_clause() {
        let diags = lint("for (let i = 0; i < 10; ) {}");
        assert!(diags.is_empty(), "no update clause should not be flagged");
    }

    #[test]
    fn test_lte_wrong_direction() {
        let diags = lint("for (let i = 0; i <= 10; i--) {}");
        assert_eq!(diags.len(), 1, "<= with decrement should be flagged");
    }

    #[test]
    fn test_gte_wrong_direction() {
        let diags = lint("for (let i = 10; i >= 0; i++) {}");
        assert_eq!(diags.len(), 1, ">= with increment should be flagged");
    }
}
