//! Rule: `no-implicit-coercion`
//!
//! Disallow shorthand type conversions. Implicit coercions like `!!x`
//! (to boolean), `+x` (to number), or `"" + x` (to string) are less
//! readable than explicit calls like `Boolean(x)`, `Number(x)`, or
//! `String(x)`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
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

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // !!x → Boolean(x)
            AstKind::UnaryExpression(outer) if outer.operator == UnaryOperator::LogicalNot => {
                if let Expression::UnaryExpression(inner) = &outer.argument {
                    if inner.operator == UnaryOperator::LogicalNot {
                        ctx.report_warning(
                            "no-implicit-coercion",
                            "Use `Boolean(x)` instead of `!!x`",
                            Span::new(outer.span.start, outer.span.end),
                        );
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
                    ctx.report_warning(
                        "no-implicit-coercion",
                        "Use `Number(x)` instead of `+x`",
                        Span::new(expr.span.start, expr.span.end),
                    );
                }
            }
            // "" + x → String(x)
            AstKind::BinaryExpression(expr) if expr.operator == BinaryOperator::Addition => {
                let left_is_empty_string = matches!(
                    &expr.left,
                    Expression::StringLiteral(s) if s.value.is_empty()
                );
                if left_is_empty_string {
                    ctx.report_warning(
                        "no-implicit-coercion",
                        "Use `String(x)` instead of `\"\" + x`",
                        Span::new(expr.span.start, expr.span.end),
                    );
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
