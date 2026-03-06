//! Rule: `no-unneeded-ternary`
//!
//! Disallow ternary operators when simpler alternatives exist.
//! `x ? true : false` can be replaced with `!!x`, and `x ? false : true`
//! can be replaced with `!x`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags ternary expressions that can be simplified to boolean coercion.
#[derive(Debug)]
pub struct NoUnneededTernary;

impl NativeRule for NoUnneededTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unneeded-ternary".to_owned(),
            description: "Disallow ternary operators when simpler alternatives exist".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ConditionalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ConditionalExpression(expr) = kind else {
            return;
        };

        let consequent_bool = as_boolean_literal(&expr.consequent);
        let alternate_bool = as_boolean_literal(&expr.alternate);

        let (Some(consequent_val), Some(alternate_val)) = (consequent_bool, alternate_bool) else {
            return;
        };

        // x ? true : true or x ? false : false — technically flaggable but
        // no-constant-condition covers these better.
        if consequent_val == alternate_val {
            return;
        }

        let test_span = expr.test.span();
        let test_start = usize::try_from(test_span.start).unwrap_or(0);
        let test_end = usize::try_from(test_span.end).unwrap_or(0);
        let Some(test_text) = ctx.source_text().get(test_start..test_end) else {
            return;
        };

        let needs_parens = !is_simple_expression(&expr.test);

        // x ? true : false → !!x (or !!(expr))
        // x ? false : true → !x (or !(expr))
        let (replacement, description) = if consequent_val {
            let inner = if needs_parens {
                format!("!!({test_text})")
            } else {
                format!("!!{test_text}")
            };
            (inner, "boolean cast")
        } else {
            let inner = if needs_parens {
                format!("!({test_text})")
            } else {
                format!("!{test_text}")
            };
            (inner, "negation")
        };

        ctx.report(Diagnostic {
            rule_name: "no-unneeded-ternary".to_owned(),
            message: format!("Unnecessary ternary — use {description} instead"),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace with `{replacement}`")),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace with `{replacement}`"),
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

/// Extract the boolean value from a `BooleanLiteral` expression.
fn as_boolean_literal(expr: &Expression<'_>) -> Option<bool> {
    if let Expression::BooleanLiteral(lit) = expr {
        Some(lit.value)
    } else {
        None
    }
}

/// Returns true if the expression is a simple identifier or literal (no parens needed).
const fn is_simple_expression(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::Identifier(_)
            | Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::ThisExpression(_)
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) else {
            return vec![];
        };
        let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnneededTernary)];
        traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
    }

    #[test]
    fn test_flags_true_false() {
        let diags = lint("const a = x ? true : false;");
        assert_eq!(diags.len(), 1, "should flag x ? true : false");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("!!x"),
            "fix should be !!x"
        );
    }

    #[test]
    fn test_flags_false_true() {
        let diags = lint("const a = x ? false : true;");
        assert_eq!(diags.len(), 1, "should flag x ? false : true");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("!x"),
            "fix should be !x"
        );
    }

    #[test]
    fn test_wraps_complex_test_in_parens() {
        let diags = lint("const a = a === b ? true : false;");
        assert_eq!(diags.len(), 1);
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("!!(a === b)"),
            "complex test should be wrapped in parens"
        );
    }

    #[test]
    fn test_ignores_non_boolean_branches() {
        let diags = lint("const a = x ? 1 : 0;");
        assert!(
            diags.is_empty(),
            "non-boolean branches should not be flagged"
        );
    }

    #[test]
    fn test_ignores_mixed_branches() {
        let diags = lint("const a = x ? true : y;");
        assert!(
            diags.is_empty(),
            "mixed boolean/non-boolean should not be flagged"
        );
    }

    #[test]
    fn test_ignores_same_boolean_branches() {
        let diags = lint("const a = x ? true : true;");
        assert!(
            diags.is_empty(),
            "same-value branches deferred to no-constant-condition"
        );
    }

    #[test]
    fn test_negation_wraps_complex_test() {
        let diags = lint("const a = a || b ? false : true;");
        assert_eq!(diags.len(), 1);
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("!(a || b)"),
            "negation with complex test should use parens"
        );
    }
}
