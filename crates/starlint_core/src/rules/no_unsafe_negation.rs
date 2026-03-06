//! Rule: `no-unsafe-negation`
//!
//! Disallow negating the left operand of relational operators. Writing
//! `!a in b` is parsed as `(!a) in b`, not `!(a in b)`. This is almost
//! always a mistake — the same applies to `instanceof`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `!x in y` and `!x instanceof y` patterns.
#[derive(Debug)]
pub struct NoUnsafeNegation;

impl NativeRule for NoUnsafeNegation {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unsafe-negation".to_owned(),
            description: "Disallow negating the left operand of relational operators".to_owned(),
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

        // Only check `in` and `instanceof` operators
        if expr.operator != BinaryOperator::In && expr.operator != BinaryOperator::Instanceof {
            return;
        }

        // Check if the left side is a `!` unary expression
        if let Expression::UnaryExpression(unary) = &expr.left {
            if unary.operator == UnaryOperator::LogicalNot {
                let op_name = if expr.operator == BinaryOperator::In {
                    "in"
                } else {
                    "instanceof"
                };

                // Fix: `!a in b` → `!(a in b)`
                #[allow(clippy::as_conversions)]
                let fix = {
                    let source = ctx.source_text();
                    let inner_span = unary.argument.span();
                    let right_span = expr.right.span();
                    let inner_text = source
                        .get(inner_span.start as usize..inner_span.end as usize)
                        .unwrap_or("");
                    let right_text = source
                        .get(right_span.start as usize..right_span.end as usize)
                        .unwrap_or("");
                    let replacement = format!("!({inner_text} {op_name} {right_text})");
                    Some(Fix {
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr.span.start, expr.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                };

                ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                    rule_name: "no-unsafe-negation".to_owned(),
                    message: format!(
                        "Unexpected negating the left operand of `{op_name}` operator"
                    ),
                    span: Span::new(expr.span.start, expr.span.end),
                    severity: Severity::Error,
                    help: Some(format!(
                        "Use `!(a {op_name} b)` instead of `!a {op_name} b`"
                    )),
                    fix,
                    labels: vec![],
                });
            }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeNegation)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_negation_in() {
        let diags = lint("if (!key in obj) {}");
        assert_eq!(diags.len(), 1, "!key in obj should be flagged");
    }

    #[test]
    fn test_flags_negation_instanceof() {
        let diags = lint("if (!obj instanceof Foo) {}");
        assert_eq!(diags.len(), 1, "!obj instanceof Foo should be flagged");
    }

    #[test]
    fn test_allows_negated_result() {
        let diags = lint("if (!(key in obj)) {}");
        assert!(diags.is_empty(), "!(key in obj) should not be flagged");
    }

    #[test]
    fn test_allows_normal_in() {
        let diags = lint("if (key in obj) {}");
        assert!(diags.is_empty(), "normal in should not be flagged");
    }

    #[test]
    fn test_allows_normal_instanceof() {
        let diags = lint("if (obj instanceof Foo) {}");
        assert!(diags.is_empty(), "normal instanceof should not be flagged");
    }
}
