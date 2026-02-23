//! Rule: `text-encoding-identifier-case` (unicorn)
//!
//! Enforce consistent casing for text encoding identifiers. Prefer `'utf-8'`
//! over `'UTF-8'`, `'utf8'`, `'Utf8'`, etc.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags non-canonical text encoding identifier casing.
#[derive(Debug)]
pub struct TextEncodingIdentifierCase;

/// Known encoding identifiers and their canonical forms.
fn canonical_encoding(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "utf-8" | "utf8" => {
            if value == "utf-8" {
                None // already canonical
            } else {
                Some("utf-8")
            }
        }
        "ascii" => {
            if value == "ascii" {
                None
            } else {
                Some("ascii")
            }
        }
        "utf-16le" | "utf16le" => {
            if value == "utf-16le" {
                None
            } else {
                Some("utf-16le")
            }
        }
        "utf-16be" | "utf16be" => {
            if value == "utf-16be" {
                None
            } else {
                Some("utf-16be")
            }
        }
        _ => None,
    }
}

impl LintRule for TextEncodingIdentifierCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "text-encoding-identifier-case".to_owned(),
            description: "Enforce consistent casing for text encoding identifiers".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
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
        if let Some(canonical) = canonical_encoding(value) {
            // Replace the string content inside the quotes (span includes quotes).
            let content_start = lit.span.start.saturating_add(1);
            let content_end = lit.span.end.saturating_sub(1);

            ctx.report(Diagnostic {
                rule_name: "text-encoding-identifier-case".to_owned(),
                message: format!("Prefer `'{canonical}'` over `'{value}'`"),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace `'{value}'` with `'{canonical}'`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace with `'{canonical}'`"),
                    edits: vec![Edit {
                        span: Span::new(content_start, content_end),
                        replacement: canonical.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(TextEncodingIdentifierCase)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_uppercase_utf8() {
        let diags = lint("const enc = 'UTF-8';");
        assert_eq!(diags.len(), 1, "UTF-8 should be flagged");
    }

    #[test]
    fn test_flags_utf8_without_hyphen() {
        let diags = lint("const enc = 'utf8';");
        assert_eq!(diags.len(), 1, "utf8 should be flagged");
    }

    #[test]
    fn test_allows_lowercase_utf8() {
        let diags = lint("const enc = 'utf-8';");
        assert!(diags.is_empty(), "utf-8 should not be flagged");
    }

    #[test]
    fn test_flags_uppercase_ascii() {
        let diags = lint("const enc = 'ASCII';");
        assert_eq!(diags.len(), 1, "ASCII should be flagged");
    }

    #[test]
    fn test_allows_lowercase_ascii() {
        let diags = lint("const enc = 'ascii';");
        assert!(diags.is_empty(), "ascii should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_string() {
        let diags = lint("const x = 'hello';");
        assert!(diags.is_empty(), "unrelated strings should not be flagged");
    }
}
