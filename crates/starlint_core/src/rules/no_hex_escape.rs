//! Rule: `no-hex-escape` (unicorn)
//!
//! Disallow hex escape sequences `\xNN` in strings — use Unicode escapes
//! `\uNNNN` instead for consistency and clarity.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags hex escape sequences in string literals.
#[derive(Debug)]
pub struct NoHexEscape;

impl LintRule for NoHexEscape {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-hex-escape".to_owned(),
            description: r"Disallow `\xNN` hex escapes — use `\uNNNN` instead".to_owned(),
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

        // Check the raw source for \x escapes
        let source = ctx.source_text();
        let start = usize::try_from(lit.span.start).unwrap_or(0);
        let end = usize::try_from(lit.span.end).unwrap_or(0);
        let Some(raw) = source.get(start..end) else {
            return;
        };

        let finding = raw.contains("\\x");
        if finding {
            let fixed = convert_hex_to_unicode(raw);

            ctx.report(Diagnostic {
                rule_name: "no-hex-escape".to_owned(),
                message: r"Use Unicode escape `\uNNNN` instead of hex escape `\xNN`".to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Warning,
                help: Some(r"Replace `\xNN` with `\u00NN`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: r"Convert hex escapes to Unicode escapes".to_owned(),
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
}

/// Convert `\xNN` hex escapes to `\u00NN` Unicode escapes in raw source.
fn convert_hex_to_unicode(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        if chars.peek() == Some(&'x') {
            let _ = chars.next(); // consume 'x'
            // Collect 2 hex digits
            let mut hex = String::new();
            for _ in 0..2 {
                if let Some(&c) = chars.peek() {
                    if c.is_ascii_hexdigit() {
                        hex.push(c);
                        let _ = chars.next();
                    } else {
                        break;
                    }
                }
            }
            if hex.len() == 2 {
                result.push_str("\\u00");
            } else {
                // Incomplete hex escape, keep as-is
                result.push('\\');
                result.push('x');
            }
            result.push_str(&hex);
        } else {
            result.push('\\');
            if let Some(&next) = chars.peek() {
                result.push(next);
                let _ = chars.next();
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoHexEscape)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_hex_escape() {
        let diags = lint(r"var s = '\x41';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_unicode_escape() {
        let diags = lint(r"var s = '\u0041';");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_normal_string() {
        let diags = lint(r"var s = 'hello';");
        assert!(diags.is_empty());
    }
}
