//! Rule: `no-implicit-coercion`
//!
//! Disallow shorthand type conversions. Implicit coercions like `!!x`
//! (to boolean), `+x` (to number), or `"" + x` (to string) are less
//! readable than explicit calls like `Boolean(x)`, `Number(x)`, or
//! `String(x)`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags implicit type coercions.
#[derive(Debug)]
pub struct NoImplicitCoercion;

impl NativeRule for NoImplicitCoercion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-implicit-coercion".to_owned(),
            description: "Disallow shorthand type conversions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression, AstType::UnaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // !!x → Boolean(x)
            AstKind::UnaryExpression(outer) if outer.operator == UnaryOperator::LogicalNot => {
                if let Expression::UnaryExpression(inner) = &outer.argument {
                    if inner.operator == UnaryOperator::LogicalNot {
                        let source = ctx.source_text();
                        let arg_start = usize::try_from(inner.argument.span().start).unwrap_or(0);
                        let arg_end = usize::try_from(inner.argument.span().end).unwrap_or(0);
                        let arg_text = source.get(arg_start..arg_end).unwrap_or("x");

                        ctx.report(Diagnostic {
                            rule_name: "no-implicit-coercion".to_owned(),
                            message: "Use `Boolean(x)` instead of `!!x`".to_owned(),
                            span: Span::new(outer.span.start, outer.span.end),
                            severity: Severity::Warning,
                            help: Some("Replace with `Boolean()`".to_owned()),
                            fix: Some(Fix {
                                message: "Replace with `Boolean()`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(outer.span.start, outer.span.end),
                                    replacement: format!("Boolean({arg_text})"),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            // +x → Number(x) (unary plus on non-numeric)
            AstKind::UnaryExpression(expr) if expr.operator == UnaryOperator::UnaryPlus => {
                // Only flag if the argument is not a numeric literal
                if !matches!(
                    &expr.argument,
                    Expression::NumericLiteral(_) | Expression::BigIntLiteral(_)
                ) {
                    let source = ctx.source_text();
                    let arg_start = usize::try_from(expr.argument.span().start).unwrap_or(0);
                    let arg_end = usize::try_from(expr.argument.span().end).unwrap_or(0);
                    let arg_text = source.get(arg_start..arg_end).unwrap_or("x");

                    ctx.report(Diagnostic {
                        rule_name: "no-implicit-coercion".to_owned(),
                        message: "Use `Number(x)` instead of `+x`".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `Number()`".to_owned()),
                        fix: Some(Fix {
                            message: "Replace with `Number()`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement: format!("Number({arg_text})"),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            // "" + x → String(x)
            AstKind::BinaryExpression(expr) if expr.operator == BinaryOperator::Addition => {
                let left_is_empty_string = matches!(
                    &expr.left,
                    Expression::StringLiteral(s) if s.value.is_empty()
                );
                if left_is_empty_string {
                    let source = ctx.source_text();
                    let right_start = usize::try_from(expr.right.span().start).unwrap_or(0);
                    let right_end = usize::try_from(expr.right.span().end).unwrap_or(0);
                    let right_text = source.get(right_start..right_end).unwrap_or("x");

                    ctx.report(Diagnostic {
                        rule_name: "no-implicit-coercion".to_owned(),
                        message: "Use `String(x)` instead of `\"\" + x`".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `String()`".to_owned()),
                        fix: Some(Fix {
                            message: "Replace with `String()`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement: format!("String({right_text})"),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoImplicitCoercion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_double_negation() {
        let diags = lint("var b = !!x;");
        assert_eq!(diags.len(), 1, "!!x should be flagged");
    }

    #[test]
    fn test_flags_unary_plus() {
        let diags = lint("var n = +x;");
        assert_eq!(diags.len(), 1, "+x should be flagged");
    }

    #[test]
    fn test_flags_empty_string_concat() {
        let diags = lint("var s = '' + x;");
        assert_eq!(diags.len(), 1, "empty string concat should be flagged");
    }

    #[test]
    fn test_allows_boolean_call() {
        let diags = lint("var b = Boolean(x);");
        assert!(diags.is_empty(), "Boolean(x) should not be flagged");
    }

    #[test]
    fn test_allows_number_call() {
        let diags = lint("var n = Number(x);");
        assert!(diags.is_empty(), "Number(x) should not be flagged");
    }

    #[test]
    fn test_allows_unary_plus_on_number() {
        let diags = lint("var n = +5;");
        assert!(
            diags.is_empty(),
            "unary plus on number literal should not be flagged"
        );
    }
}
