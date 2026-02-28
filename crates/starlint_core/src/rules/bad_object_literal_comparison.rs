//! Rule: `bad-object-literal-comparison` (OXC)
//!
//! Catch comparisons like `x === {}` or `x === []` which are always false
//! because object/array literals create new references.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags equality comparisons against object or array literals.
#[derive(Debug)]
pub struct BadObjectLiteralComparison;

impl NativeRule for BadObjectLiteralComparison {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-object-literal-comparison".to_owned(),
            description: "Catch `x === {}` or `x === []` (always false)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        if !expr.operator.is_equality() {
            return;
        }

        let left_is_literal = is_object_or_array_literal(&expr.left);
        let right_is_literal = is_object_or_array_literal(&expr.right);

        // Flag if either side is an object/array literal (and the other is not)
        if left_is_literal || right_is_literal {
            let kind_name = if left_is_literal {
                literal_kind_name(&expr.left)
            } else {
                literal_kind_name(&expr.right)
            };
            ctx.report_warning(
                "bad-object-literal-comparison",
                &format!(
                    "Comparison against {kind_name} literal is always false — \
                     object/array literals create new references"
                ),
                Span::new(expr.span.start, expr.span.end),
            );
        }
    }
}

/// Check if an expression is an object or array literal.
const fn is_object_or_array_literal(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ObjectExpression(_) | Expression::ArrayExpression(_)
    )
}

/// Get a human-readable name for the literal kind.
const fn literal_kind_name(expr: &Expression<'_>) -> &'static str {
    match expr {
        Expression::ObjectExpression(_) => "an object",
        Expression::ArrayExpression(_) => "an array",
        _ => "a literal",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BadObjectLiteralComparison)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_strict_equality() {
        let diags = lint("if (x === {}) {}");
        assert_eq!(diags.len(), 1, "x === empty object should be flagged");
    }

    #[test]
    fn test_flags_array_strict_equality() {
        let diags = lint("if (x === []) {}");
        assert_eq!(diags.len(), 1, "x === empty array should be flagged");
    }

    #[test]
    fn test_flags_loose_equality() {
        let diags = lint("if (x == {}) {}");
        assert_eq!(diags.len(), 1, "x == empty object should be flagged");
    }

    #[test]
    fn test_flags_inequality() {
        let diags = lint("if (x !== []) {}");
        assert_eq!(diags.len(), 1, "x !== empty array should be flagged");
    }

    #[test]
    fn test_allows_string_comparison() {
        let diags = lint("if (x === 'hello') {}");
        assert!(diags.is_empty(), "string comparison should not be flagged");
    }

    #[test]
    fn test_allows_variable_comparison() {
        let diags = lint("if (x === y) {}");
        assert!(
            diags.is_empty(),
            "variable comparison should not be flagged"
        );
    }
}
