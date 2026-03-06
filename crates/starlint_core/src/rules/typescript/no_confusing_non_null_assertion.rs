//! Rule: `typescript/no-confusing-non-null-assertion`
//!
//! Disallow non-null assertions (`!`) in positions where they can be confused
//! with comparison operators. Writing `x! == y` or `x! === y` is visually
//! confusing because the `!` blends with the equality operator. The reader
//! may interpret it as `x !== y` instead of `(x!) == y`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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

        if !expr.operator.is_equality() {
            return;
        }

        if let Expression::TSNonNullExpression(non_null) = &expr.left {
            // Wrap the non-null assertion in parentheses: `x! == y` → `(x!) == y`
            let source = ctx.source_text();
            let left_start = usize::try_from(non_null.span().start).unwrap_or(0);
            let left_end = usize::try_from(non_null.span().end).unwrap_or(0);
            let left_text = source.get(left_start..left_end).unwrap_or("");
            let replacement = format!("({left_text})");

            ctx.report(Diagnostic {
                rule_name: "typescript/no-confusing-non-null-assertion".to_owned(),
                message: "Non-null assertion `!` next to an equality operator is confusing — it may look like `!=` or `!==`".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some("Wrap the non-null assertion in parentheses to clarify intent".to_owned()),
                fix: Some(Fix {
                    message: "Wrap in parentheses: `(x!)`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(non_null.span().start, non_null.span().end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
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
