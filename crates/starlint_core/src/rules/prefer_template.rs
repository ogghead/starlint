//! Rule: `prefer-template`
//!
//! Suggest using template literals instead of string concatenation.
//! Template literals are more readable when combining strings with
//! variables.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags string concatenation that could use template literals.
#[derive(Debug)]
pub struct PreferTemplate;

impl NativeRule for PreferTemplate {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-template".to_owned(),
            description: "Suggest using template literals instead of string concatenation"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        if expr.operator != BinaryOperator::Addition {
            return;
        }

        // Check if this is string concatenation (at least one side is a string)
        let left_is_string = is_string_expression(&expr.left);
        let right_is_string = is_string_expression(&expr.right);

        if !left_is_string && !right_is_string {
            return;
        }

        // Don't flag if both sides are string literals (that's no-useless-concat)
        if left_is_string && right_is_string {
            return;
        }

        // Build fix: convert to template literal
        let source = ctx.source_text();
        let fix = build_template_fix(source, &expr.left, &expr.right);

        ctx.report(Diagnostic {
            rule_name: "prefer-template".to_owned(),
            message: "Unexpected string concatenation — prefer template literals".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Use a template literal instead".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Convert to template literal".to_owned(),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
                    replacement: fix,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Check if an expression is a string literal or template literal.
const fn is_string_expression(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
    )
}

/// Extract the raw content of a string literal (between quotes) from source,
/// escaping backticks and `${` for template literal context.
fn string_content_for_template(source: &str, expr: &Expression<'_>) -> Option<String> {
    let start = usize::try_from(expr.span().start).unwrap_or(0);
    let end = usize::try_from(expr.span().end).unwrap_or(0);
    let raw = source.get(start..end)?;
    // Strip surrounding quotes (single, double, or backtick)
    let inner = raw.get(1..raw.len().saturating_sub(1)).unwrap_or("");
    Some(inner.replace('`', "\\`").replace("${", "\\${"))
}

/// Build the replacement template literal string.
fn build_template_fix(source: &str, left: &Expression<'_>, right: &Expression<'_>) -> String {
    let left_start = usize::try_from(left.span().start).unwrap_or(0);
    let left_end = usize::try_from(left.span().end).unwrap_or(0);
    let right_start = usize::try_from(right.span().start).unwrap_or(0);
    let right_end = usize::try_from(right.span().end).unwrap_or(0);

    let left_str = if is_string_expression(left) {
        string_content_for_template(source, left).unwrap_or_default()
    } else {
        let text = source.get(left_start..left_end).unwrap_or("");
        format!("${{{text}}}")
    };

    let right_str = if is_string_expression(right) {
        string_content_for_template(source, right).unwrap_or_default()
    } else {
        let text = source.get(right_start..right_end).unwrap_or("");
        format!("${{{text}}}")
    };

    format!("`{left_str}{right_str}`")
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferTemplate)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_plus_variable() {
        let diags = lint("var x = 'hello ' + name;");
        assert_eq!(diags.len(), 1, "string + variable should be flagged");
    }

    #[test]
    fn test_allows_template_literal() {
        let diags = lint("var x = `hello ${name}`;");
        assert!(diags.is_empty(), "template literal should not be flagged");
    }

    #[test]
    fn test_allows_number_addition() {
        let diags = lint("var x = 1 + 2;");
        assert!(diags.is_empty(), "number addition should not be flagged");
    }

    #[test]
    fn test_allows_string_literal_concat() {
        // This is handled by no-useless-concat
        let diags = lint("var x = 'a' + 'b';");
        assert!(
            diags.is_empty(),
            "string literal concat should not be flagged by prefer-template"
        );
    }
}
