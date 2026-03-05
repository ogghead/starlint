//! Rule: `no-constant-binary-expression`
//!
//! Disallow expressions where the operation is guaranteed to produce a
//! predictable result. For example, `x === null || x === undefined` when
//! using `??` would suffice, or comparisons where one side is always a
//! new object literal (`x === {}` is always `false`).

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags binary expressions that always produce the same result.
#[derive(Debug)]
pub struct NoConstantBinaryExpression;

impl NativeRule for NoConstantBinaryExpression {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-constant-binary-expression".to_owned(),
            description: "Disallow expressions where the operation is predictable".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        // Check: comparison against a newly constructed object/array/regex
        // e.g. `x === {}`, `x === []`, `x === /re/` — always false for ===,
        // always true for !==
        if expr.operator.is_equality()
            && (is_always_new_value(&expr.left) || is_always_new_value(&expr.right))
        {
            let result_word = if expr.operator == BinaryOperator::StrictInequality
                || expr.operator == BinaryOperator::Inequality
            {
                "true"
            } else {
                "false"
            };
            ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                rule_name: "no-constant-binary-expression".to_owned(),
                message: format!(
                    "This comparison is always `{result_word}` because a new value is created each time"
                ),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression always creates a new value (object/array/regex/class).
const fn is_always_new_value(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::RegExpLiteral(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_)
    )
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstantBinaryExpression)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_comparison_with_object_literal() {
        let diags = lint("if (x === {}) {}");
        assert_eq!(
            diags.len(),
            1,
            "x === empty object should be flagged (always false)"
        );
    }

    #[test]
    fn test_flags_comparison_with_array_literal() {
        let diags = lint("if (x === []) {}");
        assert_eq!(diags.len(), 1, "x === [] should be flagged (always false)");
    }

    #[test]
    fn test_flags_comparison_with_regex() {
        let diags = lint("if (x === /re/) {}");
        assert_eq!(
            diags.len(),
            1,
            "x === /re/ should be flagged (always false)"
        );
    }

    #[test]
    fn test_flags_inequality_with_object() {
        let diags = lint("if (x !== {}) {}");
        assert_eq!(
            diags.len(),
            1,
            "x !== empty object should be flagged (always true)"
        );
    }

    #[test]
    fn test_allows_comparison_with_variable() {
        let diags = lint("if (x === y) {}");
        assert!(
            diags.is_empty(),
            "comparison with variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_comparison_with_null() {
        let diags = lint("if (x === null) {}");
        assert!(
            diags.is_empty(),
            "comparison with null should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_equality() {
        let diags = lint("var x = {} + 1;");
        assert!(
            diags.is_empty(),
            "non-equality with object should not be flagged"
        );
    }
}
