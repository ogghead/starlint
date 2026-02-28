//! Rule: `no-extra-boolean-cast`
//!
//! Disallow unnecessary boolean casts. In contexts where the result is
//! already coerced to a boolean (e.g. `if`, `while`, `for`, ternary test,
//! logical `!`), wrapping in `Boolean()` or `!!` is redundant.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary boolean casts like `!!x` in boolean contexts.
#[derive(Debug)]
pub struct NoExtraBooleanCast;

impl NativeRule for NoExtraBooleanCast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-boolean-cast".to_owned(),
            description: "Disallow unnecessary boolean casts".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Check conditions in if/while/for/ternary for double-negation or Boolean()
        let test_expr: Option<&Expression<'_>> = match kind {
            AstKind::IfStatement(stmt) => Some(&stmt.test),
            AstKind::WhileStatement(stmt) => Some(&stmt.test),
            AstKind::DoWhileStatement(stmt) => Some(&stmt.test),
            AstKind::ForStatement(stmt) => stmt.test.as_ref(),
            AstKind::ConditionalExpression(expr) => Some(&expr.test),
            _ => None,
        };

        let Some(test) = test_expr else {
            return;
        };

        if is_double_negation(test) || is_boolean_call(test) {
            ctx.report_warning(
                "no-extra-boolean-cast",
                "Redundant double negation in boolean context",
                Span::new(test.span().start, test.span().end),
            );
        }
    }
}

/// Check if expression is `!!x`.
fn is_double_negation(expr: &Expression<'_>) -> bool {
    if let Expression::UnaryExpression(outer) = expr {
        if outer.operator == UnaryOperator::LogicalNot {
            if let Expression::UnaryExpression(inner) = &outer.argument {
                return inner.operator == UnaryOperator::LogicalNot;
            }
        }
    }
    false
}

/// Check if expression is `Boolean(x)`.
fn is_boolean_call(expr: &Expression<'_>) -> bool {
    if let Expression::CallExpression(call) = expr {
        return matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "Boolean");
    }
    false
}

use oxc_span::GetSpan;

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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraBooleanCast)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_double_negation_in_if() {
        let diags = lint("if (!!x) {}");
        assert_eq!(diags.len(), 1, "!!x in if condition should be flagged");
    }

    #[test]
    fn test_flags_boolean_call_in_if() {
        let diags = lint("if (Boolean(x)) {}");
        assert_eq!(
            diags.len(),
            1,
            "Boolean(x) in if condition should be flagged"
        );
    }

    #[test]
    fn test_allows_simple_condition() {
        let diags = lint("if (x) {}");
        assert!(diags.is_empty(), "simple condition should not be flagged");
    }

    #[test]
    fn test_flags_double_negation_in_ternary() {
        let diags = lint("var r = !!x ? 1 : 0;");
        assert_eq!(diags.len(), 1, "!!x in ternary should be flagged");
    }

    #[test]
    fn test_allows_single_negation() {
        let diags = lint("if (!x) {}");
        assert!(diags.is_empty(), "single negation should not be flagged");
    }
}
