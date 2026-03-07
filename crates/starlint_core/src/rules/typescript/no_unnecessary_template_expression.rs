//! Rule: `typescript/no-unnecessary-template-expression`
//!
//! Disallow unnecessary template expressions. A template literal that contains
//! a single expression and no meaningful static text (e.g. `` `${x}` ``) is
//! unnecessary and can be replaced with the expression itself or `String(x)`.
//! Similarly, a template literal with no expressions and only static text
//! should be a regular string literal.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags template literals that are unnecessary wrappers around a single
/// expression or that contain no expressions at all.
#[derive(Debug)]
pub struct NoUnnecessaryTemplateExpression;

impl LintRule for NoUnnecessaryTemplateExpression {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-template-expression".to_owned(),
            description: "Disallow unnecessary template expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TemplateLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TemplateLiteral(template) = node else {
            return;
        };

        // A template literal with exactly one expression and no meaningful
        // static parts (quasis are all empty) is unnecessary: `${x}` -> x
        if template.expressions.len() == 1 && all_quasis_empty(&template.quasis) {
            let template_span = Span::new(template.span.start, template.span.end);
            let message = "Unnecessary template expression — use the value directly or `String(...)` instead of wrapping in a template literal";

            // Extract the expression text from source
            let source = ctx.source_text();
            let Some(&expr_id) = template.expressions.first() else {
                return;
            };
            let Some(expr_node) = ctx.node(expr_id) else {
                return;
            };
            let expr_span = expr_node.span();
            let expr_start = usize::try_from(expr_span.start).unwrap_or(0);
            let expr_end = usize::try_from(expr_span.end).unwrap_or(0);
            let expr_text = source.get(expr_start..expr_end).unwrap_or("");

            ctx.report(Diagnostic {
                rule_name: "typescript/no-unnecessary-template-expression".to_owned(),
                message: message.to_owned(),
                span: template_span,
                severity: Severity::Warning,
                help: Some("Remove the template literal wrapper".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with the expression directly".to_owned(),
                    edits: vec![Edit {
                        span: template_span,
                        replacement: expr_text.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
            return;
        }

        // A template literal with zero expressions is just a static string
        // and should use a regular string literal instead.
        if template.expressions.is_empty() && template.quasis.len() == 1 {
            // Only flag single-line static templates (multi-line templates may
            // be intentional for readability).
            let is_multiline = template.quasis.first().is_some_and(|q| q.contains('\n'));

            if !is_multiline {
                let template_span = Span::new(template.span.start, template.span.end);
                let message = "Unnecessary template literal with no expressions — use a regular string literal instead";

                // Extract the static text content and wrap in quotes
                let raw_text = template.quasis.first().map_or("", |q| q.as_str());
                // Escape any single quotes in the content for the replacement
                let escaped = raw_text.replace('\'', "\\'");
                let replacement = format!("'{escaped}'");

                ctx.report(Diagnostic {
                    rule_name: "typescript/no-unnecessary-template-expression".to_owned(),
                    message: message.to_owned(),
                    span: template_span,
                    severity: Severity::Warning,
                    help: Some("Replace with a regular string literal".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Convert to a regular string literal".to_owned(),
                        edits: vec![Edit {
                            span: template_span,
                            replacement,
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

/// Check whether all quasis (static template parts) contain only empty strings.
fn all_quasis_empty(quasis: &[String]) -> bool {
    quasis.iter().all(std::string::String::is_empty)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnnecessaryTemplateExpression)];
        lint_source(source, "test.ts", &rules)
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
