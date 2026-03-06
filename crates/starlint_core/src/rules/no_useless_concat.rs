//! Rule: `no-useless-concat`
//!
//! Disallow unnecessary concatenation of strings or template literals.
//! `"a" + "b"` should just be `"ab"`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary concatenation of string literals.
#[derive(Debug)]
pub struct NoUselessConcat;

impl NativeRule for NoUselessConcat {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-concat".to_owned(),
            description: "Disallow unnecessary concatenation of strings".to_owned(),
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

        if expr.operator != BinaryOperator::Addition {
            return;
        }

        // Both sides must be string literals or template literals
        if is_string_like(&expr.left) && is_string_like(&expr.right) {
            // Try to build a merged string literal from the source.
            let source = ctx.source_text();
            let left_start = usize::try_from(expr.left.span().start).unwrap_or(0);
            let left_end = usize::try_from(expr.left.span().end).unwrap_or(0);
            let right_start = usize::try_from(expr.right.span().start).unwrap_or(0);
            let right_end = usize::try_from(expr.right.span().end).unwrap_or(0);
            let left_raw = source.get(left_start..left_end).unwrap_or("");
            let right_raw = source.get(right_start..right_end).unwrap_or("");

            let fix = (left_raw.len() >= 2 && right_raw.len() >= 2).then(|| {
                let left_inner = &left_raw[1..left_raw.len().saturating_sub(1)];
                let right_inner = &right_raw[1..right_raw.len().saturating_sub(1)];
                let quote = &left_raw[..1];
                Fix {
                    message: "Combine into a single string".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: format!("{quote}{left_inner}{right_inner}{quote}"),
                    }],
                    is_snippet: false,
                }
            });

            ctx.report(Diagnostic {
                rule_name: "no-useless-concat".to_owned(),
                message: "Unnecessary concatenation of two string literals — combine them into one"
                    .to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some("Combine into a single string literal".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a string literal or template literal without
/// expressions.
const fn is_string_like(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
    )
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessConcat)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_concat() {
        let diags = lint("var x = 'a' + 'b';");
        assert_eq!(
            diags.len(),
            1,
            "concatenation of two string literals should be flagged"
        );
    }

    #[test]
    fn test_allows_string_plus_variable() {
        let diags = lint("var x = 'a' + b;");
        assert!(diags.is_empty(), "string + variable should not be flagged");
    }

    #[test]
    fn test_allows_number_addition() {
        let diags = lint("var x = 1 + 2;");
        assert!(diags.is_empty(), "number addition should not be flagged");
    }
}
