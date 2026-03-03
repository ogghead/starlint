//! Rule: `prefer-logical-operator-over-ternary`
//!
//! Prefer `??` / `||` over ternary when the test is a simple
//! truthiness/nullishness check. `a ? a : b` should be `a || b`, and
//! `a !== null ? a : b` / `a !== undefined ? a : b` should be `a ?? b`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags ternary expressions that can be replaced with `||` or `??`.
#[derive(Debug)]
pub struct PreferLogicalOperatorOverTernary;

impl NativeRule for PreferLogicalOperatorOverTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-logical-operator-over-ternary".to_owned(),
            description: "Prefer `??` / `||` over ternary for truthiness/nullish checks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ConditionalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ConditionalExpression(cond) = kind else {
            return;
        };

        let source = ctx.source_text();

        // Pattern 1: `a ? a : b` => `a || b`
        // The test is a simple expression and the consequent is the same expression.
        if let Some(suggestion) = check_simple_truthiness(&cond.test, &cond.consequent, source) {
            ctx.report_warning(
                "prefer-logical-operator-over-ternary",
                &format!("Use `{suggestion}` instead of a ternary expression"),
                Span::new(cond.span.start, cond.span.end),
            );
            return;
        }

        // Pattern 2: `a != null ? a : b` / `a !== null ? a : b` /
        //            `a !== undefined ? a : b` => `a ?? b`
        if check_nullish(&cond.test, &cond.consequent, source) {
            ctx.report_warning(
                "prefer-logical-operator-over-ternary",
                "Use `??` instead of a ternary expression for nullish checks",
                Span::new(cond.span.start, cond.span.end),
            );
        }
    }
}

/// Extract a slice of source text for a given expression span.
fn expr_text<'s>(expr: &Expression<'_>, source: &'s str) -> Option<&'s str> {
    let sp = expr.span();
    let start = usize::try_from(sp.start).ok()?;
    let end = usize::try_from(sp.end).ok()?;
    source.get(start..end)
}

/// Check `a ? a : b` pattern (test == consequent by source text).
fn check_simple_truthiness(
    test: &Expression<'_>,
    consequent: &Expression<'_>,
    source: &str,
) -> Option<&'static str> {
    // Skip if the test is a binary expression (those are comparisons, not simple truthiness)
    if matches!(test, Expression::BinaryExpression(_)) {
        return None;
    }

    let test_text = expr_text(test, source)?;
    let cons_text = expr_text(consequent, source)?;

    (!test_text.is_empty() && test_text == cons_text).then_some("||")
}

/// Check whether an expression is `null` or `undefined`.
fn is_nullish_literal(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::NullLiteral(_) => true,
        Expression::Identifier(id) => id.name.as_str() == "undefined",
        _ => false,
    }
}

/// Check `a !== null ? a : b` or `a !== undefined ? a : b` or `a != null ? a : b`.
fn check_nullish(test: &Expression<'_>, consequent: &Expression<'_>, source: &str) -> bool {
    let Expression::BinaryExpression(binary) = test else {
        return false;
    };

    // Must be `!==` or `!=`
    if !matches!(
        binary.operator,
        BinaryOperator::StrictInequality | BinaryOperator::Inequality
    ) {
        return false;
    }

    // Determine which side is the value and which is null/undefined
    let value_expr = if is_nullish_literal(&binary.right) {
        &binary.left
    } else if is_nullish_literal(&binary.left) {
        &binary.right
    } else {
        return false;
    };

    // The value side should match the consequent
    let Some(value_text) = expr_text(value_expr, source) else {
        return false;
    };
    let Some(cons_text) = expr_text(consequent, source) else {
        return false;
    };

    !value_text.is_empty() && value_text == cons_text
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferLogicalOperatorOverTernary)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_truthiness_ternary() {
        let diags = lint("const x = a ? a : b;");
        assert_eq!(diags.len(), 1, "a ? a : b should be flagged (use ||)");
    }

    #[test]
    fn test_flags_not_null_ternary() {
        let diags = lint("const x = a !== null ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a !== null ? a : b should be flagged (use ??)"
        );
    }

    #[test]
    fn test_flags_not_undefined_ternary() {
        let diags = lint("const x = a !== undefined ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a !== undefined ? a : b should be flagged (use ??)"
        );
    }

    #[test]
    fn test_flags_loose_not_null_ternary() {
        let diags = lint("const x = a != null ? a : b;");
        assert_eq!(
            diags.len(),
            1,
            "a != null ? a : b should be flagged (use ??)"
        );
    }

    #[test]
    fn test_allows_different_consequent() {
        let diags = lint("const x = a ? b : c;");
        assert!(
            diags.is_empty(),
            "different consequent should not be flagged"
        );
    }

    #[test]
    fn test_allows_logical_or() {
        let diags = lint("const x = a || b;");
        assert!(diags.is_empty(), "already using || should not be flagged");
    }

    #[test]
    fn test_allows_comparison_ternary() {
        let diags = lint("const x = a > 0 ? a : 0;");
        assert!(
            diags.is_empty(),
            "comparison-based ternary should not be flagged by this rule"
        );
    }
}
