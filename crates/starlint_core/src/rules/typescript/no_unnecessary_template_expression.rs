//! Rule: `typescript/no-unnecessary-template-expression`
//!
//! Disallow unnecessary template expressions. A template literal that contains
//! a single expression and no meaningful static text (e.g. `` `${x}` ``) is
//! unnecessary and can be replaced with the expression itself or `String(x)`.
//! Similarly, a template literal with no expressions and only static text
//! should be a regular string literal.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags template literals that are unnecessary wrappers around a single
/// expression or that contain no expressions at all.
#[derive(Debug)]
pub struct NoUnnecessaryTemplateExpression;

impl NativeRule for NoUnnecessaryTemplateExpression {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-template-expression".to_owned(),
            description: "Disallow unnecessary template expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TemplateLiteral(template) = kind else {
            return;
        };

        // A template literal with exactly one expression and no meaningful
        // static parts (quasis are all empty) is unnecessary: `${x}` -> x
        if template.expressions.len() == 1 && all_quasis_empty(&template.quasis) {
            ctx.report_warning(
                "typescript/no-unnecessary-template-expression",
                "Unnecessary template expression — use the value directly or `String(...)` instead of wrapping in a template literal",
                Span::new(template.span.start, template.span.end),
            );
            return;
        }

        // A template literal with zero expressions is just a static string
        // and should use a regular string literal instead.
        if template.expressions.is_empty() && template.quasis.len() == 1 {
            // Only flag single-line static templates (multi-line templates may
            // be intentional for readability).
            let is_multiline = template
                .quasis
                .first()
                .is_some_and(|q| q.value.raw.as_str().contains('\n'));

            if !is_multiline {
                ctx.report_warning(
                    "typescript/no-unnecessary-template-expression",
                    "Unnecessary template literal with no expressions — use a regular string literal instead",
                    Span::new(template.span.start, template.span.end),
                );
            }
        }
    }
}

/// Check whether all quasis (static template parts) contain only empty strings.
fn all_quasis_empty(quasis: &[oxc_ast::ast::TemplateElement<'_>]) -> bool {
    quasis.iter().all(|q| q.value.raw.is_empty())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> =
                vec![Box::new(NoUnnecessaryTemplateExpression)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_single_expression_template() {
        let diags = lint("const x = `${name}`;");
        assert_eq!(
            diags.len(),
            1,
            "template with only one expression and no static text should be flagged"
        );
    }

    #[test]
    fn test_flags_no_expression_template() {
        let diags = lint("const x = `hello`;");
        assert_eq!(
            diags.len(),
            1,
            "template literal with no expressions should be flagged"
        );
    }

    #[test]
    fn test_allows_template_with_mixed_content() {
        let diags = lint("const x = `hello ${name}`;");
        assert!(
            diags.is_empty(),
            "template with both static text and expressions should not be flagged"
        );
    }

    #[test]
    fn test_allows_template_with_multiple_expressions() {
        let diags = lint("const x = `${first} ${last}`;");
        assert!(
            diags.is_empty(),
            "template with multiple expressions should not be flagged"
        );
    }

    #[test]
    fn test_allows_multiline_template() {
        let diags = lint("const x = `hello\nworld`;");
        assert!(
            diags.is_empty(),
            "multiline template literal should not be flagged"
        );
    }
}
