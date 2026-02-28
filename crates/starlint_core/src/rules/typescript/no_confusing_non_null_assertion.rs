//! Rule: `typescript/no-confusing-non-null-assertion`
//!
//! Disallow non-null assertions (`!`) in positions where they can be confused
//! with comparison operators. Writing `x! == y` or `x! === y` is visually
//! confusing because the `!` blends with the equality operator. The reader
//! may interpret it as `x !== y` instead of `(x!) == y`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags binary equality expressions where the left operand is a
/// `TSNonNullExpression`, making the `!` look like part of `!=` or `!==`.
#[derive(Debug)]
pub struct NoConfusingNonNullAssertion;

impl NativeRule for NoConfusingNonNullAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-confusing-non-null-assertion".to_owned(),
            description:
                "Disallow non-null assertions in confusing positions next to equality operators"
                    .to_owned(),
            category: Category::Style,
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

        if matches!(&expr.left, Expression::TSNonNullExpression(_)) {
            ctx.report_warning(
                "typescript/no-confusing-non-null-assertion",
                "Non-null assertion `!` next to an equality operator is confusing — it may look like `!=` or `!==`",
                Span::new(expr.span.start, expr.span.end),
            );
        }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConfusingNonNullAssertion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_non_null_before_equality() {
        let diags = lint("declare const x: number | null; x! == 1;");
        assert_eq!(diags.len(), 1, "`x! == 1` should be flagged as confusing");
    }

    #[test]
    fn test_flags_non_null_before_strict_equality() {
        let diags = lint("declare const x: number | null; x! === 1;");
        assert_eq!(diags.len(), 1, "`x! === 1` should be flagged as confusing");
    }

    #[test]
    fn test_flags_non_null_before_inequality() {
        let diags = lint("declare const x: number | null; x! != 1;");
        assert_eq!(diags.len(), 1, "`x! != 1` should be flagged as confusing");
    }

    #[test]
    fn test_allows_normal_equality() {
        let diags = lint("const x = 1; x == 1;");
        assert!(diags.is_empty(), "normal equality should not be flagged");
    }

    #[test]
    fn test_allows_strict_inequality() {
        let diags = lint("const x = 1; x !== null;");
        assert!(
            diags.is_empty(),
            "normal strict inequality should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_null_in_non_equality() {
        let diags = lint("declare const x: number | null; const y = x! + 1;");
        assert!(
            diags.is_empty(),
            "non-null assertion with arithmetic should not be flagged"
        );
    }
}
