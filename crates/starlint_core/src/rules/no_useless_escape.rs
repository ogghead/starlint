//! Rule: `no-useless-escape`
//!
//! Disallow unnecessary escape characters in strings and regular expressions.
//! Characters that don't need escaping produce unnecessary visual noise.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags unnecessary escape characters in string literals.
#[derive(Debug)]
pub struct NoUselessEscape;

/// Check if a character needs escaping in a string literal.
const fn is_meaningful_escape(ch: char) -> bool {
    matches!(
        ch,
        '\\' | 'n' | 'r' | 't' | 'b' | 'f' | 'v' | 'u' | '0' | '\'' | '"' | '`' | '\n' | '\r'
    ) || ch.is_ascii_digit()
        || ch == 'x'
}

impl LintRule for NoUselessEscape {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-escape".to_owned(),
            description: "Disallow unnecessary escape characters".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StringLiteral])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StringLiteral(lit) = node else {
            return;
        };

        let source = ctx.source_text();
        let raw = source
            .get(lit.span.start as usize..lit.span.end as usize)
            .unwrap_or("");

        // Need at least 2 chars for opening and closing quotes
        if raw.len() < 2 {
            return;
        }

        // Strip the quote characters (first and last)
        let inner = &raw[1..raw.len().saturating_sub(1)];

        if !has_useless_escape(inner) {
            return;
        }

        let quote = &raw[..1];
        let fixed_inner = strip_useless_escapes(inner);
        let fixed = format!("{quote}{fixed_inner}{quote}");

        ctx.report(Diagnostic {
            rule_name: "no-useless-escape".to_owned(),
            message: "Unnecessary escape character".to_owned(),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: Some("Remove the unnecessary escape".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove unnecessary escape".to_owned(),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement: fixed,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Remove useless backslash escapes from inner string content (without quotes).
fn strip_useless_escapes(inner: &str) -> String {
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    loop {
        match chars.next() {
            None => return result,
            Some('\\') => match chars.next() {
                None => {
                    result.push('\\');
                    return result;
                }
                Some(next_ch) => {
                    if is_meaningful_escape(next_ch) {
                        result.push('\\');
                    }
                    result.push(next_ch);
                }
            },
            Some(ch) => result.push(ch),
        }
    }
}

/// Scan inner string content (without quotes) for useless backslash escapes.
fn has_useless_escape(inner: &str) -> bool {
    let mut chars = inner.chars();
    loop {
        match chars.next() {
            None => return false,
            Some('\\') => match chars.next() {
                None => return false,
                Some(next_ch) => {
                    if !is_meaningful_escape(next_ch) {
                        return true;
                    }
                }
            },
            Some(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessEscape)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_useless_escape() {
        let diags = lint(r#"var x = "hell\o";"#);
        assert!(!diags.is_empty(), "useless escape of 'o' should be flagged");
    }

    #[test]
    fn test_allows_needed_escape() {
        let diags = lint(r#"var x = "hello\nworld";"#);
        assert!(diags.is_empty(), "newline escape should not be flagged");
    }

    #[test]
    fn test_allows_quote_escape() {
        let diags = lint(r#"var x = "it\'s";"#);
        assert!(diags.is_empty(), "quote escape should not be flagged");
    }
}
