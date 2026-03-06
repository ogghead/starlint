//! Rule: `typescript/no-unnecessary-type-assertion`
//!
//! Flags `x as T` type assertions where the expression is obviously already of
//! type `T`. Without full type inference we detect obvious literal cases:
//! string literal `as string`, number literal `as number`, boolean literal
//! `as boolean`, `null as null`, and `undefined as undefined`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, TSType};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `as T` assertions that are unnecessary because the expression already
/// matches the asserted type.
#[derive(Debug)]
pub struct NoUnnecessaryTypeAssertion;

impl NativeRule for NoUnnecessaryTypeAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-type-assertion".to_owned(),
            description: "Disallow type assertions that do not change the type of an expression"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSAsExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSAsExpression(expr) = kind else {
            return;
        };

        if let Some(description) = is_unnecessary_assertion(&expr.expression, &expr.type_annotation)
        {
            // Fix: replace `expr as T` with just `expr`
            let inner_span = expr.expression.span();
            let inner_text =
                ctx.source_text()[inner_span.start as usize..inner_span.end as usize].to_owned();

            ctx.report(Diagnostic {
                rule_name: "typescript/no-unnecessary-type-assertion".to_owned(),
                message: format!("Unnecessary type assertion: {description}"),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some(format!(
                    "Remove the `as` assertion — replace with `{inner_text}`"
                )),
                fix: Some(Fix {
                    message: "Remove unnecessary type assertion".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: inner_text,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check whether an `as T` assertion is unnecessary because the expression
/// already has the asserted type.
///
/// Returns a human-readable description when the assertion is unnecessary,
/// or `None` when it is (potentially) meaningful.
fn is_unnecessary_assertion<'a>(
    expression: &Expression<'a>,
    type_annotation: &TSType<'a>,
) -> Option<&'static str> {
    match (expression, type_annotation) {
        (Expression::StringLiteral(_), TSType::TSStringKeyword(_)) => {
            Some("string literal is already of type `string`")
        }
        (Expression::NumericLiteral(_), TSType::TSNumberKeyword(_)) => {
            Some("number literal is already of type `number`")
        }
        (Expression::BooleanLiteral(_), TSType::TSBooleanKeyword(_)) => {
            Some("boolean literal is already of type `boolean`")
        }
        (Expression::NullLiteral(_), TSType::TSNullKeyword(_)) => {
            Some("`null` is already of type `null`")
        }
        (Expression::Identifier(ident), TSType::TSUndefinedKeyword(_))
            if ident.name == "undefined" =>
        {
            Some("`undefined` is already of type `undefined`")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnnecessaryTypeAssertion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_literal_as_string() {
        let diags = lint(r#"let x = "hello" as string;"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal asserted as string should be flagged"
        );
    }

    #[test]
    fn test_flags_number_literal_as_number() {
        let diags = lint("let x = 42 as number;");
        assert_eq!(
            diags.len(),
            1,
            "number literal asserted as number should be flagged"
        );
    }

    #[test]
    fn test_flags_boolean_literal_as_boolean() {
        let diags = lint("let x = true as boolean;");
        assert_eq!(
            diags.len(),
            1,
            "boolean literal asserted as boolean should be flagged"
        );
    }

    #[test]
    fn test_flags_null_as_null() {
        let diags = lint("let x = null as null;");
        assert_eq!(diags.len(), 1, "`null as null` should be flagged");
    }

    #[test]
    fn test_allows_meaningful_assertion() {
        let diags = lint("let x = value as string;");
        assert!(
            diags.is_empty(),
            "assertion of non-literal to a type should not be flagged"
        );
    }
}
