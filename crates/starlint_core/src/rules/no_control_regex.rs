//! Rule: `no-control-regex`
//!
//! Disallow control characters in regular expressions. Control characters
//! (ASCII 0x01-0x1F) are rarely useful in regex patterns and are usually
//! a mistake.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags regular expression literals that contain control characters.
#[derive(Debug)]
pub struct NoControlRegex;

impl NativeRule for NoControlRegex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-control-regex".to_owned(),
            description: "Disallow control characters in regular expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::RegExpLiteral(regex) = kind else {
            return;
        };

        let pattern = regex.regex.pattern.text.as_str();

        if has_control_character(pattern) {
            ctx.report_error(
                "no-control-regex",
                "Unexpected control character in regular expression",
                Span::new(regex.span.start, regex.span.end),
            );
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code with the `NoControlRegex` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoControlRegex)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
