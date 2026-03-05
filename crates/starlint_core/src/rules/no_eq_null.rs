//! Rule: `no-eq-null`
//!
//! Disallow `null` comparisons without type-checking operators.
//! `x == null` should use `x === null` or `x === undefined` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags loose equality comparisons with `null`.
#[derive(Debug)]
pub struct NoEqNull;

impl NativeRule for NoEqNull {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-eq-null".to_owned(),
            description: "Disallow `null` comparisons without type-checking operators".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        // Only check loose equality (== and !=)
        if expr.operator != BinaryOperator::Equality && expr.operator != BinaryOperator::Inequality
        {
            return;
        }

        let has_null = is_null(&expr.left) || is_null(&expr.right);

        if has_null {
            let source = ctx.source_text();
            let left_end = usize::try_from(expr.left.span().end).unwrap_or(0);
            let right_start = usize::try_from(expr.right.span().start).unwrap_or(0);
            let between = source.get(left_end..right_start).unwrap_or("");
            let op_str = if expr.operator == BinaryOperator::Equality {
                "=="
            } else {
                "!="
            };
            let replacement_op = if expr.operator == BinaryOperator::Equality {
                "==="
            } else {
                "!=="
            };

            let fix = between.find(op_str).map(|offset| {
                let op_pos = u32::try_from(left_end.saturating_add(offset)).unwrap_or(0);
                Fix {
                    message: format!("Replace `{op_str}` with `{replacement_op}`"),
                    edits: vec![Edit {
                        span: Span::new(op_pos, op_pos.saturating_add(2)),
                        replacement: replacement_op.to_owned(),
                    }],
                }
            });

            ctx.report(Diagnostic {
                rule_name: "no-eq-null".to_owned(),
                message: "Use `===` or `!==` to compare with `null`".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace `{op_str}` with `{replacement_op}`")),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is `null`.
const fn is_null(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::NullLiteral(_))
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEqNull)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_loose_equality_null() {
        let diags = lint("if (x == null) {}");
        assert_eq!(diags.len(), 1, "x == null should be flagged");
    }

    #[test]
    fn test_flags_loose_inequality_null() {
        let diags = lint("if (x != null) {}");
        assert_eq!(diags.len(), 1, "x != null should be flagged");
    }

    #[test]
    fn test_allows_strict_equality_null() {
        let diags = lint("if (x === null) {}");
        assert!(diags.is_empty(), "x === null should not be flagged");
    }

    #[test]
    fn test_allows_strict_inequality_null() {
        let diags = lint("if (x !== null) {}");
        assert!(diags.is_empty(), "x !== null should not be flagged");
    }
}
