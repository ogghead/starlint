//! Rule: `no-template-curly-in-string`
//!
//! Disallow template literal placeholder syntax in regular strings.
//! Writing `"Hello ${name}"` instead of `` `Hello ${name}` `` is a common
//! mistake — the `${...}` is treated as literal text, not as interpolation.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags regular string literals that contain `${...}` template syntax.
#[derive(Debug)]
pub struct NoTemplateCurlyInString;

impl LintRule for NoTemplateCurlyInString {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-template-curly-in-string".to_owned(),
            description: "Disallow template literal placeholder syntax in regular strings"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StringLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StringLiteral(lit) = node else {
            return;
        };

        let value = lit.value.as_str();
        if contains_template_placeholder(value) {
            // Fix: convert string quotes to backticks to make a template literal
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let start = lit.span.start as usize;
                let end = lit.span.end as usize;
                source.get(start..end).and_then(|raw| {
                    // Get the raw source content (without surrounding quotes)
                    let inner = raw.get(1..raw.len().saturating_sub(1))?;
                    // Escape any backticks in the content
                    let escaped = inner.replace('`', "\\`");
                    let replacement = format!("`{escaped}`");
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Convert to template literal".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(lit.span.start, lit.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                })
            };

            ctx.report(Diagnostic {
                rule_name: "no-template-curly-in-string".to_owned(),
                message: "Unexpected template string expression in a regular string".to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Error,
                help: Some(
                    "Use a template literal (backticks) instead of a regular string".to_owned(),
                ),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if a string contains what looks like a template placeholder `${...}`.
fn contains_template_placeholder(s: &str) -> bool {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes.get(i).copied() == Some(b'$') {
            let next = i.checked_add(1).unwrap_or(len);
            if bytes.get(next).copied() == Some(b'{') {
                // Look for the closing brace
                let search_start = next.checked_add(1).unwrap_or(len);
                let mut j = search_start;
                while j < len {
                    if bytes.get(j).copied() == Some(b'}') {
                        return true;
                    }
                    j = j.checked_add(1).unwrap_or(len);
                }
            }
        }
        i = i.checked_add(1).unwrap_or(len);
    }
    false
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoTemplateCurlyInString)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_template_in_double_quotes() {
        let diags = lint(r#"var x = "Hello ${name}";"#);
        assert_eq!(
            diags.len(),
            1,
            "template placeholder in string should be flagged"
        );
    }

    #[test]
    fn test_flags_template_in_single_quotes() {
        let diags = lint("var x = 'Hello ${name}';");
        assert_eq!(
            diags.len(),
            1,
            "template placeholder in single-quoted string should be flagged"
        );
    }

    #[test]
    fn test_allows_template_literal() {
        let diags = lint("var x = `Hello ${name}`;");
        assert!(diags.is_empty(), "template literal should not be flagged");
    }

    #[test]
    fn test_allows_dollar_without_brace() {
        let diags = lint(r#"var x = "price is $5";"#);
        assert!(diags.is_empty(), "$5 without braces should not be flagged");
    }

    #[test]
    fn test_allows_plain_string() {
        let diags = lint(r#"var x = "hello world";"#);
        assert!(diags.is_empty(), "plain string should not be flagged");
    }

    #[test]
    fn test_flags_expression_template() {
        let diags = lint(r#"var x = "result: ${a + b}";"#);
        assert_eq!(
            diags.len(),
            1,
            "template expression in string should be flagged"
        );
    }

    #[test]
    fn test_allows_unclosed_placeholder() {
        let diags = lint(r#"var x = "price: ${unclosed";"#);
        assert!(
            diags.is_empty(),
            "unclosed placeholder should not be flagged"
        );
    }
}
