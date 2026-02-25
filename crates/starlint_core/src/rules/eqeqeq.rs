//! Rule: `eqeqeq`
//!
//! Require `===` and `!==` instead of `==` and `!=`.
//! The loose equality operators perform type coercion which is a common
//! source of bugs.

use oxc_ast::AstKind;
use oxc_ast::ast::BinaryOperator;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `==` and `!=` operators, suggesting `===` and `!==` instead.
#[derive(Debug)]
pub struct Eqeqeq;

impl NativeRule for Eqeqeq {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "eqeqeq".to_owned(),
            description: "Require `===` and `!==`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::BinaryExpression(expr) = kind {
            let (replacement, label) = match expr.operator {
                BinaryOperator::Equality => ("===", "=="),
                BinaryOperator::Inequality => ("!==", "!="),
                _ => return,
            };

            // Search only between the operands to avoid matching operators
            // inside string literals (e.g., `"a == b" == x`).
            let op_span = find_operator_span(
                ctx.source_text(),
                expr.left.span().end,
                expr.right.span().start,
                label,
            );

            ctx.report(Diagnostic {
                rule_name: "eqeqeq".to_owned(),
                message: format!("Expected `{replacement}` and instead saw `{label}`"),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: Some(format!("Use `{replacement}` instead of `{label}`")),
                fix: Some(Fix {
                    message: format!("Replace `{label}` with `{replacement}`"),
                    edits: vec![Edit {
                        span: op_span,
                        replacement: replacement.to_owned(),
                    }],
                }),
                labels: vec![],
            });
        }
    }
}

/// Find the span of the operator within a binary expression.
///
/// Searches the source text between `start` and `end` for the operator string.
/// Falls back to the full expression span if not found.
fn find_operator_span(source: &str, start: u32, end: u32, operator: &str) -> Span {
    let source_len = u32::try_from(source.len()).unwrap_or(u32::MAX);
    let clamped_start = usize::try_from(start.min(source_len)).unwrap_or(0);
    let clamped_end = usize::try_from(end.min(source_len)).unwrap_or(0);

    if let Some(slice) = source.get(clamped_start..clamped_end) {
        if let Some(offset) = slice.find(operator) {
            let op_start = start.saturating_add(u32::try_from(offset).unwrap_or(0));
            let op_end = op_start.saturating_add(u32::try_from(operator.len()).unwrap_or(0));
            return Span::new(op_start, op_end);
        }
    }

    Span::new(start, end)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    #[test]
    fn test_flags_loose_equality() {
        let allocator = Allocator::default();
        let source = "if (a == b) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Eqeqeq)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag == operator");
            let first = diags.first();
            assert!(
                first.is_some_and(|d| d.fix.is_some()),
                "should provide a fix"
            );
        }
    }

    #[test]
    fn test_flags_loose_inequality() {
        let allocator = Allocator::default();
        let source = "if (a != b) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Eqeqeq)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag != operator");
        }
    }

    #[test]
    fn test_fix_targets_operator_not_string_content() {
        // Regression: `"a == b" == x` must fix the operator between
        // the string literal and `x`, not the `==` inside the string.
        let allocator = Allocator::default();
        let source = r#"if ("a == b" == x) {}"#;
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Eqeqeq)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag == operator");
            if let Some(diag) = diags.first() {
                if let Some(fix) = &diag.fix {
                    if let Some(edit) = fix.edits.first() {
                        let start = usize::try_from(edit.span.start).unwrap_or(0);
                        let end = usize::try_from(edit.span.end).unwrap_or(0);
                        let fixed_slice = source.get(start..end).unwrap_or("");
                        assert_eq!(
                            fixed_slice, "==",
                            "fix span should target the actual operator"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_allows_strict_equality() {
        let allocator = Allocator::default();
        let source = "if (a === b && c !== d) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Eqeqeq)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(diags.is_empty(), "strict equality should not be flagged");
        }
    }
}
