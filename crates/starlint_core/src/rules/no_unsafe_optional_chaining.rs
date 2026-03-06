//! Rule: `no-unsafe-optional-chaining`
//!
//! Disallow use of optional chaining in contexts where `undefined` is not
//! allowed. Using `?.` in arithmetic, `new`, destructuring, or template
//! tags can cause runtime errors because the result might be `undefined`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unsafe uses of optional chaining that could produce undefined
/// in contexts where it causes errors.
#[derive(Debug)]
pub struct NoUnsafeOptionalChaining;

impl NativeRule for NoUnsafeOptionalChaining {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unsafe-optional-chaining".to_owned(),
            description:
                "Disallow use of optional chaining in contexts where undefined is not allowed"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::BinaryExpression,
            AstType::NewExpression,
            AstType::SpreadElement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // `new foo?.bar()` — undefined is not a constructor
            AstKind::NewExpression(new_expr) => {
                if contains_optional_chain(&new_expr.callee) {
                    ctx.report(Diagnostic {
                        rule_name: "no-unsafe-optional-chaining".to_owned(),
                        message: "Unsafe use of optional chaining in `new` expression".to_owned(),
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            // Arithmetic operations on optional chain: `foo?.bar + 1`
            AstKind::BinaryExpression(bin) => {
                if bin.operator.is_arithmetic() || bin.operator.is_bitwise() {
                    if contains_optional_chain(&bin.left) {
                        report_arithmetic(bin.span, ctx);
                    }
                    if contains_optional_chain(&bin.right) {
                        report_arithmetic(bin.span, ctx);
                    }
                }
            }
            // Spread: `[...foo?.bar]` — undefined is not iterable
            AstKind::SpreadElement(spread) => {
                if contains_optional_chain(&spread.argument) {
                    ctx.report(Diagnostic {
                        rule_name: "no-unsafe-optional-chaining".to_owned(),
                        message: "Unsafe use of optional chaining in spread element".to_owned(),
                        span: Span::new(spread.span.start, spread.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Check if an expression directly contains optional chaining.
fn contains_optional_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ChainExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => contains_optional_chain(&paren.expression),
        _ => false,
    }
}

/// Report unsafe optional chaining in arithmetic context.
fn report_arithmetic(span: oxc_span::Span, ctx: &mut NativeLintContext<'_>) {
    ctx.report(Diagnostic {
        rule_name: "no-unsafe-optional-chaining".to_owned(),
        message: "Unsafe use of optional chaining in arithmetic operation".to_owned(),
        span: Span::new(span.start, span.end),
        severity: Severity::Error,
        help: None,
        fix: None,
        labels: vec![],
    });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeOptionalChaining)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_with_optional_chain() {
        let diags = lint("new (foo?.bar)();");
        assert_eq!(diags.len(), 1, "new with optional chain should be flagged");
    }

    #[test]
    fn test_flags_arithmetic_with_optional_chain() {
        let diags = lint("var x = foo?.bar + 1;");
        assert_eq!(
            diags.len(),
            1,
            "arithmetic with optional chain should be flagged"
        );
    }

    #[test]
    fn test_allows_safe_optional_chain() {
        let diags = lint("var x = foo?.bar;");
        assert!(
            diags.is_empty(),
            "simple optional chain should not be flagged"
        );
    }

    #[test]
    fn test_allows_optional_chain_in_condition() {
        let diags = lint("if (foo?.bar) {}");
        assert!(
            diags.is_empty(),
            "optional chain in condition should not be flagged"
        );
    }

    #[test]
    fn test_allows_nullish_coalescing() {
        let diags = lint("var x = (foo?.bar ?? 0) + 1;");
        assert!(
            diags.is_empty(),
            "optional chain with nullish coalescing should not be flagged"
        );
    }
}
