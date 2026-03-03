//! Rule: `bad-char-at-comparison` (OXC)
//!
//! Catch comparisons of `.charAt()` result against a multi-character string.
//! `.charAt()` always returns a single character (or empty string), so
//! comparing it to a multi-character string is always false.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.charAt()` compared to a multi-character string.
#[derive(Debug)]
pub struct BadCharAtComparison;

impl NativeRule for BadCharAtComparison {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-char-at-comparison".to_owned(),
            description: "Catch `.charAt()` compared to a multi-character string".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
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

        // Check left.charAt() == "multi-char" or "multi-char" == right.charAt()
        let flagged = (is_char_at_call(&expr.left) && is_multi_char_string(&expr.right))
            || (is_multi_char_string(&expr.left) && is_char_at_call(&expr.right));

        if flagged {
            ctx.report_warning(
                "bad-char-at-comparison",
                "`.charAt()` returns a single character — comparing to a multi-character \
                 string is always false",
                Span::new(expr.span.start, expr.span.end),
            );
        }
    }
}

/// Check if an expression is a `.charAt()` call.
fn is_char_at_call(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::CallExpression(call) if matches!(
            &call.callee,
            Expression::StaticMemberExpression(member) if member.property.name.as_str() == "charAt"
        )
    )
}

/// Check if an expression is a string literal with more than one character.
fn is_multi_char_string(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(s) if s.value.len() > 1
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BadCharAtComparison)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_char_at_vs_multi_char() {
        let diags = lint("if (s.charAt(0) === 'ab') {}");
        assert_eq!(
            diags.len(),
            1,
            "charAt compared to multi-char string should be flagged"
        );
    }

    #[test]
    fn test_flags_reverse_order() {
        let diags = lint("if ('ab' === s.charAt(0)) {}");
        assert_eq!(
            diags.len(),
            1,
            "multi-char string compared to charAt should be flagged"
        );
    }

    #[test]
    fn test_allows_single_char_comparison() {
        let diags = lint("if (s.charAt(0) === 'a') {}");
        assert!(
            diags.is_empty(),
            "charAt compared to single char should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_char_at_call() {
        let diags = lint("if (s.indexOf('ab') === 'ab') {}");
        assert!(diags.is_empty(), "non-charAt call should not be flagged");
    }
}
