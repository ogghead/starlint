//! Rule: `no-typeof-undefined`
//!
//! Prefer `x === undefined` over `typeof x === 'undefined'`. The `typeof`
//! guard is only needed for undeclared variables, which is rare in modern
//! module-based code.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `typeof x === 'undefined'` comparisons.
#[derive(Debug)]
pub struct NoTypeofUndefined;

impl NativeRule for NoTypeofUndefined {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-typeof-undefined".to_owned(),
            description: "Prefer direct `undefined` comparison over `typeof` check".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        // Only match equality/inequality operators.
        let new_op = match expr.operator {
            BinaryOperator::StrictEquality | BinaryOperator::Equality => "===",
            BinaryOperator::StrictInequality | BinaryOperator::Inequality => "!==",
            _ => return,
        };

        // Check both orderings: `typeof x === 'undefined'` and `'undefined' === typeof x`.
        let typeof_arg_span = match (&expr.left, &expr.right) {
            (Expression::UnaryExpression(unary), Expression::StringLiteral(lit))
                if unary.operator == UnaryOperator::Typeof && lit.value == "undefined" =>
            {
                unary.argument.span()
            }
            (Expression::StringLiteral(lit), Expression::UnaryExpression(unary))
                if unary.operator == UnaryOperator::Typeof && lit.value == "undefined" =>
            {
                unary.argument.span()
            }
            _ => return,
        };

        // Extract the argument text from source.
        let arg_start = usize::try_from(typeof_arg_span.start).unwrap_or(0);
        let arg_end = usize::try_from(typeof_arg_span.end).unwrap_or(0);
        let Some(arg_text) = ctx.source_text().get(arg_start..arg_end) else {
            return;
        };

        let replacement = format!("{arg_text} {new_op} undefined");

        ctx.report(Diagnostic {
            rule_name: "no-typeof-undefined".to_owned(),
            message: format!("Use `{replacement}` instead of `typeof` check"),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Direct `undefined` comparison is clearer in modern code".to_owned()),
            fix: Some(Fix {
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
                    replacement,
                }],
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTypeofUndefined)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_typeof_strict_equals() {
        let diags = lint("if (typeof x === 'undefined') {}");
        assert_eq!(diags.len(), 1, "should flag typeof x === 'undefined'");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x === undefined"),
            "fix should replace with direct comparison"
        );
    }

    #[test]
    fn test_flags_typeof_strict_not_equals() {
        let diags = lint("if (typeof x !== 'undefined') {}");
        assert_eq!(diags.len(), 1, "should flag typeof x !== 'undefined'");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x !== undefined"),
            "fix should use !=="
        );
    }

    #[test]
    fn test_flags_reversed_order() {
        let diags = lint("if ('undefined' === typeof x) {}");
        assert_eq!(diags.len(), 1, "should flag reversed order");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x === undefined"),
            "fix should normalize to standard order"
        );
    }

    #[test]
    fn test_flags_loose_equals() {
        let diags = lint("if (typeof x == 'undefined') {}");
        assert_eq!(diags.len(), 1, "should flag loose equality");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x === undefined"),
            "fix should upgrade to strict equality"
        );
    }

    #[test]
    fn test_flags_member_expression_arg() {
        let diags = lint("if (typeof obj.prop === 'undefined') {}");
        assert_eq!(diags.len(), 1, "should handle member expression arg");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("obj.prop === undefined"),
            "fix should preserve member expression"
        );
    }

    #[test]
    fn test_allows_typeof_string() {
        let diags = lint("if (typeof x === 'string') {}");
        assert!(
            diags.is_empty(),
            "typeof x === 'string' should not be flagged"
        );
    }

    #[test]
    fn test_allows_direct_undefined() {
        let diags = lint("if (x === undefined) {}");
        assert!(
            diags.is_empty(),
            "direct undefined comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_typeof_number() {
        let diags = lint("if (typeof x === 'number') {}");
        assert!(
            diags.is_empty(),
            "typeof x === 'number' should not be flagged"
        );
    }
}
