//! Rule: `no-negated-condition`
//!
//! Disallow negated conditions in `if` statements with an `else` branch
//! and in ternary operators. These are harder to read and should be
//! inverted for clarity.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags negated conditions that should be inverted.
#[derive(Debug)]
pub struct NoNegatedCondition;

impl NativeRule for NoNegatedCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-negated-condition".to_owned(),
            description: "Disallow negated conditions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ConditionalExpression, AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::IfStatement(stmt) => {
                // Only flag if there is an else branch
                if stmt.alternate.is_none() {
                    return;
                }
                if is_negated(&stmt.test) {
                    ctx.report_warning(
                        "no-negated-condition",
                        "Unexpected negated condition in `if` with `else`",
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }
            }
            AstKind::ConditionalExpression(expr) => {
                if is_negated(&expr.test) {
                    ctx.report_warning(
                        "no-negated-condition",
                        "Unexpected negated condition in ternary",
                        Span::new(expr.span.start, expr.span.end),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Check if an expression is a negation (`!x`) or inequality (`a !== b`, `a != b`).
fn is_negated(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary) => {
            matches!(
                binary.operator,
                oxc_ast::ast::BinaryOperator::StrictInequality
                    | oxc_ast::ast::BinaryOperator::Inequality
            )
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNegatedCondition)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_negated_if_with_else() {
        let diags = lint("if (!x) { a(); } else { b(); }");
        assert_eq!(diags.len(), 1, "negated if with else should be flagged");
    }

    #[test]
    fn test_allows_negated_if_without_else() {
        let diags = lint("if (!x) { a(); }");
        assert!(
            diags.is_empty(),
            "negated if without else should not be flagged"
        );
    }

    #[test]
    fn test_flags_negated_ternary() {
        let diags = lint("var r = !x ? a : b;");
        assert_eq!(diags.len(), 1, "negated ternary should be flagged");
    }

    #[test]
    fn test_allows_non_negated_ternary() {
        let diags = lint("var r = x ? a : b;");
        assert!(
            diags.is_empty(),
            "non-negated ternary should not be flagged"
        );
    }

    #[test]
    fn test_flags_inequality_if_with_else() {
        let diags = lint("if (a !== b) { x(); } else { y(); }");
        assert_eq!(diags.len(), 1, "inequality if with else should be flagged");
    }
}
