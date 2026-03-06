//! Rule: `consistent-existence-index-check`
//!
//! Enforce consistent style for checking if an index exists. Prefer
//! `!== -1` over `>= 0` and `=== -1` over `< 0` when checking the
//! result of `indexOf` or `findIndex`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Method names that return an index (`-1` means not found).
const INDEX_METHODS: &[&str] = &["indexOf", "findIndex"];

/// Flags inconsistent index-existence comparisons.
#[derive(Debug)]
pub struct ConsistentExistenceIndexCheck;

impl NativeRule for ConsistentExistenceIndexCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-existence-index-check".to_owned(),
            description: "Enforce consistent style for checking if an index exists".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        // Check pattern: `someCall(...) OP value`
        // where someCall is .indexOf() or .findIndex()
        if !is_index_call(&expr.left) {
            return;
        }

        // (message, fix_description, replacement_operator, replacement_value)
        let fix_info = match expr.operator {
            // `indexOf(x) >= 0` → prefer `indexOf(x) !== -1`
            BinaryOperator::GreaterEqualThan if is_numeric_literal(&expr.right, 0.0) => Some((
                "Use `!== -1` instead of `>= 0` for index existence check",
                "Replace `>= 0` with `!== -1`",
                "!==",
                "-1",
            )),
            // `indexOf(x) > -1` → prefer `indexOf(x) !== -1`
            BinaryOperator::GreaterThan if is_numeric_literal(&expr.right, -1.0) => Some((
                "Use `!== -1` instead of `> -1` for index existence check",
                "Replace `> -1` with `!== -1`",
                "!==",
                "-1",
            )),
            // `indexOf(x) < 0` → prefer `indexOf(x) === -1`
            BinaryOperator::LessThan if is_numeric_literal(&expr.right, 0.0) => Some((
                "Use `=== -1` instead of `< 0` for index non-existence check",
                "Replace `< 0` with `=== -1`",
                "===",
                "-1",
            )),
            _ => None,
        };

        if let Some((message, fix_desc, new_op, new_val)) = fix_info {
            // Find the operator span between left and right expressions
            let left_end = expr.left.span().end;
            let right_end = expr.right.span().end;

            ctx.report(Diagnostic {
                rule_name: "consistent-existence-index-check".to_owned(),
                message: message.to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some(fix_desc.to_owned()),
                fix: Some(Fix {
                    message: fix_desc.to_owned(),
                    edits: vec![Edit {
                        span: Span::new(left_end, right_end),
                        replacement: format!(" {new_op} {new_val}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a call to `.indexOf()` or `.findIndex()`.
fn is_index_call(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };

    let method_name = member.property.name.as_str();
    INDEX_METHODS.contains(&method_name)
}

/// Check if an expression is a numeric literal with a specific value.
fn is_numeric_literal(expr: &Expression<'_>, value: f64) -> bool {
    // Handle negative numbers: `-1` is parsed as `UnaryExpression(-, 1)`
    if let Expression::UnaryExpression(unary) = expr {
        if unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation {
            if let Expression::NumericLiteral(lit) = &unary.argument {
                return ((-lit.value) - value).abs() < f64::EPSILON;
            }
        }
        return false;
    }

    let Expression::NumericLiteral(lit) = expr else {
        return false;
    };
    (lit.value - value).abs() < f64::EPSILON
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentExistenceIndexCheck)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_gte_zero() {
        let diags = lint("if (arr.indexOf(x) >= 0) {}");
        assert_eq!(diags.len(), 1, "`>= 0` index check should be flagged");
    }

    #[test]
    fn test_allows_not_equals_neg_one() {
        let diags = lint("if (arr.indexOf(x) !== -1) {}");
        assert!(
            diags.is_empty(),
            "`!== -1` index check should not be flagged"
        );
    }

    #[test]
    fn test_flags_gt_neg_one() {
        let diags = lint("if (arr.findIndex(x => x > 0) > -1) {}");
        assert_eq!(diags.len(), 1, "`> -1` index check should be flagged");
    }

    #[test]
    fn test_allows_equals_neg_one() {
        let diags = lint("if (str.indexOf('a') === -1) {}");
        assert!(
            diags.is_empty(),
            "`=== -1` index check should not be flagged"
        );
    }

    #[test]
    fn test_flags_lt_zero() {
        let diags = lint("if (arr.indexOf(x) < 0) {}");
        assert_eq!(diags.len(), 1, "`< 0` index check should be flagged");
    }

    #[test]
    fn test_allows_unrelated_comparison() {
        let diags = lint("if (arr.length >= 0) {}");
        assert!(
            diags.is_empty(),
            "non-indexOf comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_equals_zero() {
        let diags = lint("if (arr.indexOf(x) === 0) {}");
        assert!(diags.is_empty(), "`=== 0` is a valid specific-index check");
    }
}
