//! Rule: `prefer-string-starts-ends-with` (unicorn)
//!
//! Prefer `String#startsWith()` and `String#endsWith()` over regex tests
//! or manual index checks. For example, `/^foo/.test(str)` should be
//! `str.startsWith('foo')` and `str.indexOf('x') === 0` should be
//! `str.startsWith('x')`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags patterns that can use `startsWith`/`endsWith`.
#[derive(Debug)]
pub struct PreferStringStartsEndsWith;

impl NativeRule for PreferStringStartsEndsWith {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-starts-ends-with".to_owned(),
            description: "Prefer `startsWith()` and `endsWith()` over alternatives".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::BinaryExpression(expr) => check_index_of_comparison(expr, ctx),
            AstKind::CallExpression(call) => check_regex_test(call, ctx),
            _ => {}
        }
    }
}

/// Check for `.indexOf(x) === 0` pattern.
fn check_index_of_comparison(
    expr: &oxc_ast::ast::BinaryExpression<'_>,
    ctx: &mut NativeLintContext<'_>,
) {
    // Match: str.indexOf(x) === 0 or 0 === str.indexOf(x)
    let call = match (&expr.left, &expr.right) {
        (Expression::CallExpression(c), other) | (other, Expression::CallExpression(c))
            if is_zero_literal(other) =>
        {
            c
        }
        _ => return,
    };

    if !matches!(
        expr.operator,
        BinaryOperator::StrictEquality | BinaryOperator::Equality
    ) {
        return;
    }

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };

    if member.property.name.as_str() != "indexOf" {
        return;
    }

    if call.arguments.len() != 1 {
        return;
    }

    ctx.report_warning(
        "prefer-string-starts-ends-with",
        "Prefer `startsWith()` over `.indexOf() === 0`",
        Span::new(expr.span.start, expr.span.end),
    );
}

/// Check for `/^foo/.test(str)` or `/foo$/.test(str)` pattern.
fn check_regex_test(call: &oxc_ast::ast::CallExpression<'_>, ctx: &mut NativeLintContext<'_>) {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };

    if member.property.name.as_str() != "test" {
        return;
    }

    let Expression::RegExpLiteral(regex) = &member.object else {
        return;
    };

    let pattern = regex.regex.pattern.text.as_str();

    let (kind, literal_part) = if let Some(rest) = pattern.strip_prefix('^') {
        ("startsWith", rest)
    } else if let Some(rest) = pattern.strip_suffix('$') {
        ("endsWith", rest)
    } else {
        return;
    };

    // Only flag if the literal part is a simple string (no regex metacharacters)
    if literal_part.chars().any(|c| {
        matches!(
            c,
            '.' | '*' | '+' | '?' | '[' | ']' | '(' | ')' | '{' | '}' | '|' | '\\'
        )
    }) {
        return;
    }

    if literal_part.is_empty() {
        return;
    }

    ctx.report_warning(
        "prefer-string-starts-ends-with",
        &format!("Prefer `.{kind}('{literal_part}')` over regex test"),
        Span::new(call.span.start, call.span.end),
    );
}

/// Check if an expression is the numeric literal `0`.
fn is_zero_literal(expr: &Expression<'_>) -> bool {
    if let Expression::NumericLiteral(lit) = expr {
        return lit.value.abs() < f64::EPSILON;
    }
    false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStringStartsEndsWith)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_index_of_equals_zero() {
        let diags = lint("if (str.indexOf('foo') === 0) {}");
        assert_eq!(diags.len(), 1, "indexOf === 0 should be flagged");
    }

    #[test]
    fn test_flags_regex_starts_with() {
        let diags = lint("if (/^foo/.test(str)) {}");
        assert_eq!(diags.len(), 1, "/^foo/.test should be flagged");
    }

    #[test]
    fn test_flags_regex_ends_with() {
        let diags = lint("if (/bar$/.test(str)) {}");
        assert_eq!(diags.len(), 1, "/bar$/.test should be flagged");
    }

    #[test]
    fn test_allows_starts_with() {
        let diags = lint("if (str.startsWith('foo')) {}");
        assert!(diags.is_empty(), "startsWith should not be flagged");
    }

    #[test]
    fn test_allows_ends_with() {
        let diags = lint("if (str.endsWith('bar')) {}");
        assert!(diags.is_empty(), "endsWith should not be flagged");
    }

    #[test]
    fn test_allows_complex_regex() {
        let diags = lint("if (/^foo.*bar/.test(str)) {}");
        assert!(diags.is_empty(), "complex regex should not be flagged");
    }

    #[test]
    fn test_allows_index_of_not_zero() {
        let diags = lint("if (str.indexOf('foo') === 3) {}");
        assert!(
            diags.is_empty(),
            "indexOf !== 0 comparison should not be flagged"
        );
    }
}
