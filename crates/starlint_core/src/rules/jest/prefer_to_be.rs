//! Rule: `jest/prefer-to-be`
//!
//! Suggest `expect(x).toBe(y)` over `expect(x).toEqual(y)` for primitive
//! literal values. `toBe` uses `Object.is` which is more appropriate and
//! faster for primitives than the deep-equality check of `toEqual`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(x).toEqual(primitive)` patterns that should use `toBe`.
#[derive(Debug)]
pub struct PreferToBe;

impl NativeRule for PreferToBe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-be".to_owned(),
            description: "Suggest using `toBe()` for primitive literals instead of `toEqual()`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check that callee is a member expression with `.toEqual`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name.as_str() != "toEqual" {
            return;
        }

        // The object should be an `expect(...)` call (or chained `.not.toEqual`)
        if !is_expect_chain(&member.object) {
            return;
        }

        // Check that the first argument to `toEqual` is a primitive literal
        let Some(first_arg) = call.arguments.first() else {
            return;
        };
        let arg_expr = first_arg.as_expression();
        let Some(expr) = arg_expr else {
            return;
        };
        if is_primitive_literal(expr) {
            let prop_span = Span::new(member.property.span.start, member.property.span.end);
            ctx.report(Diagnostic {
                rule_name: "jest/prefer-to-be".to_owned(),
                message: "Use `toBe` instead of `toEqual` when comparing primitive values"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace `toEqual` with `toBe`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with `toBe`".to_owned(),
                    edits: vec![Edit {
                        span: prop_span,
                        replacement: "toBe".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check whether an expression is a primitive literal (string, number, boolean,
/// null, undefined, bigint).
fn is_primitive_literal(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
    ) || is_undefined(expr)
}

/// Check if the expression is the identifier `undefined`.
fn is_undefined(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(id) if id.name.as_str() == "undefined")
}

/// Check if an expression is an `expect(...)` call or a chain like
/// `expect(...).not`.
fn is_expect_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "expect")
        }
        Expression::StaticMemberExpression(member) => is_expect_chain(&member.object),
        _ => false,
    }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToBe)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_equal_with_number() {
        let diags = lint("expect(x).toEqual(1);");
        assert_eq!(
            diags.len(),
            1,
            "`toEqual(1)` should be flagged as prefer `toBe`"
        );
    }

    #[test]
    fn test_flags_to_equal_with_string() {
        let diags = lint(r#"expect(x).toEqual("hello");"#);
        assert_eq!(
            diags.len(),
            1,
            "`toEqual(\"hello\")` should be flagged as prefer `toBe`"
        );
    }

    #[test]
    fn test_allows_to_equal_with_object() {
        let diags = lint("expect(x).toEqual({ a: 1 });");
        assert!(
            diags.is_empty(),
            "`toEqual` with an object literal should not be flagged"
        );
    }
}
