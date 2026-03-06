//! Rule: `typescript/no-unnecessary-boolean-literal-compare`
//!
//! Disallow unnecessary equality comparisons against boolean literals.
//! Comparisons like `x === true` or `x === false` are redundant when `x`
//! is already a boolean. Prefer `x` or `!x` for cleaner, more idiomatic code.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unnecessary-boolean-literal-compare";

/// Flags comparisons where one side is a boolean literal and the operator
/// is `==`, `===`, `!=`, or `!==`.
#[derive(Debug)]
pub struct NoUnnecessaryBooleanLiteralCompare;

impl NativeRule for NoUnnecessaryBooleanLiteralCompare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary comparisons against boolean literals".to_owned(),
            category: Category::Suggestion,
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

        // Only check equality operators
        if !matches!(
            expr.operator,
            BinaryOperator::Equality
                | BinaryOperator::StrictEquality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictInequality
        ) {
            return;
        }

        let left_is_bool = is_boolean_literal(&expr.left);
        let right_is_bool = is_boolean_literal(&expr.right);

        if left_is_bool || right_is_bool {
            let op_str = match expr.operator {
                BinaryOperator::Equality => "==",
                BinaryOperator::StrictEquality => "===",
                BinaryOperator::Inequality => "!=",
                BinaryOperator::StrictInequality => "!==",
                _ => return,
            };

            let bool_val = if left_is_bool {
                boolean_value(&expr.left)
            } else {
                boolean_value(&expr.right)
            };

            let bool_str = if bool_val.unwrap_or(true) {
                "true"
            } else {
                "false"
            };

            // Build fix: determine if we need negation
            // === true / == true → just the non-bool operand
            // !== true / != true → negate the non-bool operand
            // === false / == false → negate the non-bool operand
            // !== false / != false → just the non-bool operand
            let is_equality = matches!(
                expr.operator,
                BinaryOperator::Equality | BinaryOperator::StrictEquality
            );
            let needs_negation = if bool_val.unwrap_or(true) {
                !is_equality // `!== true` or `!= true` → negate
            } else {
                is_equality // `=== false` or `== false` → negate
            };

            let source = ctx.source_text();
            let other = if left_is_bool {
                &expr.right
            } else {
                &expr.left
            };
            let other_start = usize::try_from(other.span().start).unwrap_or(0);
            let other_end = usize::try_from(other.span().end).unwrap_or(0);
            let other_text = source.get(other_start..other_end).unwrap_or("");

            let replacement = if needs_negation {
                format!("!{other_text}")
            } else {
                other_text.to_owned()
            };

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Unnecessary comparison to `{bool_str}` — simplify the expression by removing `{op_str} {bool_str}`"
                ),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some("Simplify the boolean comparison".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Simplify the boolean comparison".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a boolean literal (`true` or `false`).
const fn is_boolean_literal(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::BooleanLiteral(_))
}

/// Extract the boolean value from a boolean literal expression.
fn boolean_value(expr: &Expression<'_>) -> Option<bool> {
    if let Expression::BooleanLiteral(lit) = expr {
        Some(lit.value)
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> =
                vec![Box::new(NoUnnecessaryBooleanLiteralCompare)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_strict_equals_true() {
        let diags = lint("if (x === true) {}");
        assert_eq!(diags.len(), 1, "x === true should be flagged");
    }

    #[test]
    fn test_flags_strict_equals_false() {
        let diags = lint("if (x === false) {}");
        assert_eq!(diags.len(), 1, "x === false should be flagged");
    }

    #[test]
    fn test_flags_loose_equals_true() {
        let diags = lint("if (x == true) {}");
        assert_eq!(diags.len(), 1, "x == true should be flagged");
    }

    #[test]
    fn test_flags_not_equals_false() {
        let diags = lint("if (x !== false) {}");
        assert_eq!(diags.len(), 1, "x !== false should be flagged");
    }

    #[test]
    fn test_allows_comparison_without_boolean_literal() {
        let diags = lint("if (x === y) {}");
        assert!(
            diags.is_empty(),
            "comparison without boolean literal should not be flagged"
        );
    }
}
