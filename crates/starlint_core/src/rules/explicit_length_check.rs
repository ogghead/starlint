//! Rule: `explicit-length-check`
//!
//! Require explicit comparison when checking `.length` or `.size`.
//! Truthy/falsy checks on `.length` are confusing because `0` is falsy
//! but is a valid length. Prefer `arr.length > 0` or `arr.length === 0`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Property names that should be compared explicitly.
const LENGTH_PROPERTIES: &[&str] = &["length", "size"];

/// Flags implicit truthy/falsy checks on `.length` or `.size`.
#[derive(Debug)]
pub struct ExplicitLengthCheck;

impl NativeRule for ExplicitLengthCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "explicit-length-check".to_owned(),
            description: "Require explicit comparison when checking `.length` or `.size`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let (test_expr, container_span) = match kind {
            AstKind::IfStatement(stmt) => (&stmt.test, stmt.span),
            AstKind::WhileStatement(stmt) => (&stmt.test, stmt.span),
            AstKind::ConditionalExpression(expr) => (&expr.test, expr.span),
            _ => return,
        };

        check_condition(test_expr, container_span, ctx);
    }
}

/// Check a condition expression for implicit `.length`/`.size` usage.
fn check_condition(
    expr: &Expression<'_>,
    container_span: oxc_span::Span,
    ctx: &mut NativeLintContext<'_>,
) {
    // Case 1: `if (foo.length)` — direct member expression as condition
    if is_length_or_size_member(expr) {
        ctx.report_warning(
            "explicit-length-check",
            "Use an explicit comparison (`> 0` or `=== 0`) instead of a truthy check on `.length`/`.size`",
            Span::new(container_span.start, container_span.end),
        );
        return;
    }

    // Case 2: `if (!foo.length)` — negated member expression
    if let Expression::UnaryExpression(unary) = expr {
        if unary.operator == UnaryOperator::LogicalNot && is_length_or_size_member(&unary.argument)
        {
            ctx.report_warning(
                "explicit-length-check",
                "Use `=== 0` instead of negating `.length`/`.size`",
                Span::new(container_span.start, container_span.end),
            );
        }
    }
}

/// Check if an expression is a static member access to `.length` or `.size`.
fn is_length_or_size_member(expr: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = expr else {
        return false;
    };
    let name = member.property.name.as_str();
    LENGTH_PROPERTIES.contains(&name)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExplicitLengthCheck)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_truthy_length() {
        let diags = lint("if (arr.length) {}");
        assert_eq!(diags.len(), 1, "truthy .length check should be flagged");
    }

    #[test]
    fn test_allows_explicit_greater_than() {
        let diags = lint("if (arr.length > 0) {}");
        assert!(
            diags.is_empty(),
            "explicit > 0 comparison should not be flagged"
        );
    }

    #[test]
    fn test_flags_negated_length() {
        let diags = lint("if (!arr.length) {}");
        assert_eq!(diags.len(), 1, "negated .length check should be flagged");
    }

    #[test]
    fn test_allows_explicit_equals_zero() {
        let diags = lint("if (arr.length === 0) {}");
        assert!(
            diags.is_empty(),
            "explicit === 0 comparison should not be flagged"
        );
    }

    #[test]
    fn test_flags_truthy_size() {
        let diags = lint("if (map.size) {}");
        assert_eq!(diags.len(), 1, "truthy .size check should be flagged");
    }

    #[test]
    fn test_allows_not_equals_zero() {
        let diags = lint("if (str.length !== 0) {}");
        assert!(
            diags.is_empty(),
            "explicit !== 0 comparison should not be flagged"
        );
    }

    #[test]
    fn test_flags_while_truthy_length() {
        let diags = lint("while (arr.length) {}");
        assert_eq!(diags.len(), 1, "truthy .length in while should be flagged");
    }

    #[test]
    fn test_flags_ternary_truthy_length() {
        let diags = lint("var x = arr.length ? 'yes' : 'no';");
        assert_eq!(
            diags.len(),
            1,
            "truthy .length in ternary should be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_property() {
        let diags = lint("if (arr.count) {}");
        assert!(diags.is_empty(), "unrelated property should not be flagged");
    }
}
