//! Rule: `no-inline-comments`
//!
//! Disallow inline comments after code. Comments should appear on
//! their own line for better readability and consistency.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags comments that appear on the same line as code.
#[derive(Debug)]
pub struct NoInlineComments;

impl NativeRule for NoInlineComments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-inline-comments".to_owned(),
            description: "Disallow inline comments after code".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        // Collect violations first to avoid borrow conflict with ctx
        let violations: Vec<(u32, u32)> = {
            let source = ctx.source_text();
            let mut byte_offset: u32 = 0;
            let mut found = Vec::new();

            for line in source.lines() {
                let trimmed = line.trim();
                let line_len = u32::try_from(line.len()).unwrap_or(0);

                // Skip lines that are entirely comments or empty
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
                    continue;
                }

                if let Some(comment_pos) = find_inline_comment(line) {
                    let comment_start =
                        byte_offset.saturating_add(u32::try_from(comment_pos).unwrap_or(0));
                    let comment_end = byte_offset.saturating_add(line_len);
                    found.push((comment_start, comment_end));
                }

                byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
            }
            found
        };

        for (start, end) in violations {
            ctx.report(Diagnostic {
                rule_name: "no-inline-comments".to_owned(),
                message: "Unexpected comment inline with code".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Find the position of an inline comment (// or /*) in a line.
/// Returns `None` if the comment is at the start of the line (not inline)
/// or there is no comment. Attempts to skip over string literals.
#[allow(clippy::arithmetic_side_effects)]
fn find_inline_comment(line: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_template = false;
    let mut skip_next = false;

    let bytes = line.as_bytes();

    for (i, &ch) in bytes.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        // Backslash escapes next character inside strings
        if ch == b'\\' && (in_single_quote || in_double_quote || in_template) {
            skip_next = true;
            continue;
        }

        if in_single_quote {
            if ch == b'\'' {
                in_single_quote = false;
            }
            continue;
        }

        if in_double_quote {
            if ch == b'"' {
                in_double_quote = false;
            }
            continue;
        }

        if in_template {
            if ch == b'`' {
                in_template = false;
            }
            continue;
        }

        match ch {
            b'\'' => in_single_quote = true,
            b'"' => in_double_quote = true,
            b'`' => in_template = true,
            b'/' => {
                let next = bytes.get(i + 1).copied();
                if next == Some(b'/') || next == Some(b'*') {
                    // Check that there's non-whitespace code before this position
                    let before = line.get(..i).unwrap_or("");
                    if !before.trim().is_empty() {
                        return Some(i);
                    }
                    // If only whitespace before, this is a full-line comment
                    return None;
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInlineComments)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_inline_comment() {
        let diags = lint("var x = 1; // comment");
        assert_eq!(diags.len(), 1, "inline comment should be flagged");
    }

    #[test]
    fn test_allows_own_line_comment() {
        let diags = lint("// comment\nvar x = 1;");
        assert!(
            diags.is_empty(),
            "comment on own line should not be flagged"
        );
    }

    #[test]
    fn test_flags_inline_block_comment() {
        let diags = lint("var x = 1; /* comment */");
        assert_eq!(diags.len(), 1, "inline block comment should be flagged");
    }

    #[test]
    fn test_allows_block_comment_own_line() {
        let diags = lint("/* comment */\nvar x = 1;");
        assert!(
            diags.is_empty(),
            "block comment on own line should not be flagged"
        );
    }

    #[test]
    fn test_ignores_slash_in_string() {
        let diags = lint("var x = \"http://example.com\";");
        assert!(diags.is_empty(), "// inside string should not be flagged");
    }
}
