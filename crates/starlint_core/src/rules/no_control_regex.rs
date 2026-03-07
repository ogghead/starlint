//! Rule: `no-control-regex`
//!
//! Disallow control characters in regular expressions. Control characters
//! (ASCII 0x01-0x1F) are rarely useful in regex patterns and are usually
//! a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags regular expression literals that contain control characters.
#[derive(Debug)]
pub struct NoControlRegex;

impl LintRule for NoControlRegex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-control-regex".to_owned(),
            description: "Disallow control characters in regular expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::RegExpLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::RegExpLiteral(regex) = node else {
            return;
        };

        let pattern = regex.pattern.as_str();

        if has_control_character(pattern) {
            ctx.report(Diagnostic {
                rule_name: "no-control-regex".to_owned(),
                message: "Unexpected control character in regular expression".to_owned(),
                span: Span::new(regex.span.start, regex.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a regex pattern contains control characters or `\x00`-`\x1f`
/// escape sequences.
fn has_control_character(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes.get(i).copied();

        // Check for literal control characters (0x01-0x1f, but not 0x09/0x0a/0x0d
        // which are tab/newline/carriage-return — these wouldn't normally appear in
        // regex literals parsed by oxc)
        if let Some(b) = ch {
            if b > 0 && b < 0x20 && b != b'\t' && b != b'\n' && b != b'\r' {
                return true;
            }
        }

        // Check for \x00 through \x1f hex escape
        if ch == Some(b'\\') {
            let next = bytes.get(i.saturating_add(1)).copied();
            if next == Some(b'x') {
                // \xHH — check if HH is 00-1f
                if let Some(val) = parse_two_hex_digits(bytes, i.saturating_add(2)) {
                    if val < 0x20 {
                        return true;
                    }
                }
                i = i.saturating_add(4);
                continue;
            }
            // Skip any other escaped character
            i = i.saturating_add(2);
            continue;
        }

        i = i.saturating_add(1);
    }

    false
}

/// Parse two hex digits at position `pos` from a byte slice.
fn parse_two_hex_digits(bytes: &[u8], pos: usize) -> Option<u8> {
    let h1 = hex_value(bytes.get(pos).copied()?)?;
    let h2 = hex_value(bytes.get(pos.saturating_add(1)).copied()?)?;
    Some(h1.wrapping_mul(16).wrapping_add(h2))
}

/// Convert a hex character to its numeric value.
const fn hex_value(ch: u8) -> Option<u8> {
    match ch {
        b'0'..=b'9' => Some(ch.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(ch.wrapping_sub(b'a').wrapping_add(10)),
        b'A'..=b'F' => Some(ch.wrapping_sub(b'A').wrapping_add(10)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    /// Helper to lint source code with the `NoControlRegex` rule.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoControlRegex)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_hex_control_char() {
        let diags = lint("var re = /\\x1f/;");
        assert_eq!(diags.len(), 1, "hex control character should be flagged");
    }

    #[test]
    fn test_flags_hex_null() {
        let diags = lint("var re = /\\x00/;");
        assert_eq!(diags.len(), 1, "null hex escape should be flagged");
    }

    #[test]
    fn test_allows_normal_regex() {
        let diags = lint("var re = /foo/;");
        assert!(diags.is_empty(), "normal regex should not be flagged");
    }

    #[test]
    fn test_allows_printable_hex() {
        let diags = lint("var re = /\\x20/;");
        assert!(
            diags.is_empty(),
            "printable hex escape (space) should not be flagged"
        );
    }

    #[test]
    fn test_allows_hex_letter() {
        let diags = lint("var re = /\\x41/;");
        assert!(
            diags.is_empty(),
            "printable hex escape (A) should not be flagged"
        );
    }
}
