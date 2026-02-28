//! Rule: `no-useless-concat`
//!
//! Disallow unnecessary concatenation of strings or template literals.
//! `"a" + "b"` should just be `"ab"`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
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

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        if expr.operator != BinaryOperator::Addition {
            return;
        }

        // Both sides must be string literals or template literals
        if is_string_like(&expr.left) && is_string_like(&expr.right) {
            ctx.report_warning(
                "no-useless-concat",
                "Unnecessary concatenation of two string literals — combine them into one",
                Span::new(expr.span.start, expr.span.end),
            );
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
