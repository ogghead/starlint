//! Rule: `prefer-string-starts-ends-with` (unicorn)
//!
//! Prefer `String#startsWith()` and `String#endsWith()` over regex tests
//! or manual index checks. For example, `/^foo/.test(str)` should be
//! `str.startsWith('foo')` and `str.indexOf('x') === 0` should be
//! `str.startsWith('x')`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression, AstType::CallExpression])
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

    // Build fix: `obj.startsWith(arg)`
    let source = ctx.source_text();
    let obj_start = usize::try_from(member.object.span().start).unwrap_or(0);
    let obj_end = usize::try_from(member.object.span().end).unwrap_or(0);
    let obj_text = source.get(obj_start..obj_end).unwrap_or("");
    let arg_span = call.arguments.first().map(oxc_span::GetSpan::span);
    let fix = arg_span.and_then(|a| {
        let a_start = usize::try_from(a.start).unwrap_or(0);
        let a_end = usize::try_from(a.end).unwrap_or(0);
        let arg_text = source.get(a_start..a_end)?;
        Some(Fix {
            message: "Replace with `.startsWith()`".to_owned(),
            edits: vec![Edit {
                span: Span::new(expr.span.start, expr.span.end),
                replacement: format!("{obj_text}.startsWith({arg_text})"),
            }],
            is_snippet: false,
        })
    });

    ctx.report(Diagnostic {
        rule_name: "prefer-string-starts-ends-with".to_owned(),
        message: "Prefer `startsWith()` over `.indexOf() === 0`".to_owned(),
        span: Span::new(expr.span.start, expr.span.end),
        severity: Severity::Warning,
        help: Some("Replace with `.startsWith()`".to_owned()),
        fix,
        labels: vec![],
    });
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

    // Build fix: `str.startsWith('literal')` or `str.endsWith('literal')`
    let source = ctx.source_text();
    let fix = call.arguments.first().map(|arg| {
        let a_start = usize::try_from(arg.span().start).unwrap_or(0);
        let a_end = usize::try_from(arg.span().end).unwrap_or(0);
        let arg_text = source.get(a_start..a_end).unwrap_or("");
        Fix {
            message: format!("Replace with `.{kind}('{literal_part}')`"),
            edits: vec![Edit {
                span: Span::new(call.span.start, call.span.end),
                replacement: format!("{arg_text}.{kind}('{literal_part}')"),
            }],
            is_snippet: false,
        }
    });

    ctx.report(Diagnostic {
        rule_name: "prefer-string-starts-ends-with".to_owned(),
        message: format!("Prefer `.{kind}('{literal_part}')` over regex test"),
        span: Span::new(call.span.start, call.span.end),
        severity: Severity::Warning,
        help: Some(format!("Replace with `.{kind}('{literal_part}')`")),
        fix,
        labels: vec![],
    });
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
