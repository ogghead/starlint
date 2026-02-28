//! Rule: `expiring-todo-comments` (unicorn)
//!
//! Flags `TODO`/`FIXME` comments that contain an expiration date in the past.
//! Pattern: `TODO [YYYY-MM-DD]` or `TODO (YYYY-MM-DD)` or `FIXME [YYYY-MM-DD]`.
//! Dates before `2026-01-01` are considered expired.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Cutoff year -- dates before this date are considered expired.
const EXPIRY_YEAR: u32 = 2026;

/// Cutoff month (February).
const EXPIRY_MONTH: u32 = 2;

/// Cutoff day.
const EXPIRY_DAY: u32 = 27;

/// Flags `TODO`/`FIXME` comments with expired dates.
#[derive(Debug)]
pub struct ExpiringTodoComments;

impl NativeRule for ExpiringTodoComments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "expiring-todo-comments".to_owned(),
            description: "Flag TODO/FIXME comments with expired dates".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_expired_todos(source);

        for (message, span) in findings {
            ctx.report_warning("expiring-todo-comments", &message, span);
        }
    }
}

/// Scan source text for comments containing expired `TODO`/`FIXME` dates.
/// Returns (message, span) pairs for each expired comment found.
fn find_expired_todos(source: &str) -> Vec<(String, Span)> {
    let mut results = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos: usize = 0;

    while pos < len {
        let Some(&current) = bytes.get(pos) else {
            break;
        };

        if current == b'/' {
            let Some(&next) = bytes.get(pos.saturating_add(1)) else {
                break;
            };

            if next == b'/' {
                // Single-line comment: find end of line.
                let start = pos;
                let mut end = pos.saturating_add(2);
                while end < len {
                    if bytes.get(end).copied() == Some(b'\n') {
                        break;
                    }
                    end = end.saturating_add(1);
                }
                check_comment_for_expired_todo(source, start, end, &mut results);
                pos = end;
                continue;
            } else if next == b'*' {
                // Multi-line comment: find `*/`.
                let start = pos;
                let mut end = pos.saturating_add(2);
                loop {
                    if end.saturating_add(1) >= len {
                        end = len;
                        break;
                    }
                    if let (Some(&c1), Some(&c2)) =
                        (bytes.get(end), bytes.get(end.saturating_add(1)))
                    {
                        if c1 == b'*' && c2 == b'/' {
                            end = end.saturating_add(2);
                            break;
                        }
                    }
                    end = end.saturating_add(1);
                }
                check_comment_for_expired_todo(source, start, end, &mut results);
                pos = end;
                continue;
            }
        }

        // Skip string literals to avoid false positives.
        if current == b'"' || current == b'\'' || current == b'`' {
            let quote = current;
            pos = pos.saturating_add(1);
            while pos < len {
                let Some(&ch) = bytes.get(pos) else {
                    break;
                };
                if ch == b'\\' {
                    pos = pos.saturating_add(2);
                    continue;
                }
                if ch == quote {
                    pos = pos.saturating_add(1);
                    break;
                }
                pos = pos.saturating_add(1);
            }
            continue;
        }

        pos = pos.saturating_add(1);
    }

    results
}

/// Check a comment region for expired TODO/FIXME dates.
fn check_comment_for_expired_todo(
    source: &str,
    start: usize,
    end: usize,
    results: &mut Vec<(String, Span)>,
) {
    let comment = source.get(start..end).unwrap_or("");
    let upper = comment.to_ascii_uppercase();

    // Look for TODO or FIXME keywords.
    let has_todo = upper.contains("TODO");
    let has_fixme = upper.contains("FIXME");

    if !has_todo && !has_fixme {
        return;
    }

    // Search for date patterns: YYYY-MM-DD
    if let Some(date) = find_date_in_text(comment) {
        if is_expired(date.0, date.1, date.2) {
            let keyword = if has_todo { "TODO" } else { "FIXME" };
            let message = format!(
                "Expired {keyword} comment: date {}-{:02}-{:02} has passed",
                date.0, date.1, date.2
            );
            if let (Ok(s), Ok(e)) = (u32::try_from(start), u32::try_from(end)) {
                results.push((message, Span::new(s, e)));
            }
        }
    }
}

/// Search for a date pattern `YYYY-MM-DD` in text and return (year, month, day).
fn find_date_in_text(text: &str) -> Option<(u32, u32, u32)> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    // Need at least 10 chars for YYYY-MM-DD.
    if len < 10 {
        return None;
    }
    let search_end = len.saturating_sub(10);
    let mut i: usize = 0;

    while i <= search_end {
        // Check for 4 digits, hyphen, 2 digits, hyphen, 2 digits.
        if is_digit_at(bytes, i)
            && is_digit_at(bytes, i.saturating_add(1))
            && is_digit_at(bytes, i.saturating_add(2))
            && is_digit_at(bytes, i.saturating_add(3))
            && bytes.get(i.saturating_add(4)).copied() == Some(b'-')
            && is_digit_at(bytes, i.saturating_add(5))
            && is_digit_at(bytes, i.saturating_add(6))
            && bytes.get(i.saturating_add(7)).copied() == Some(b'-')
            && is_digit_at(bytes, i.saturating_add(8))
            && is_digit_at(bytes, i.saturating_add(9))
        {
            let date_str = text.get(i..i.saturating_add(10)).unwrap_or("");
            if let Some(date) = parse_date(date_str) {
                return Some(date);
            }
        }
        i = i.saturating_add(1);
    }

    None
}

/// Check if byte at position is an ASCII digit.
fn is_digit_at(bytes: &[u8], pos: usize) -> bool {
    bytes.get(pos).copied().is_some_and(|b| b.is_ascii_digit())
}

/// Parse a `YYYY-MM-DD` string into (year, month, day).
fn parse_date(s: &str) -> Option<(u32, u32, u32)> {
    // We know the format is exactly 10 chars: YYYY-MM-DD
    let year_str = s.get(..4)?;
    let month_str = s.get(5..7)?;
    let day_str = s.get(8..10)?;

    let year = year_str.parse::<u32>().ok()?;
    let month = month_str.parse::<u32>().ok()?;
    let day = day_str.parse::<u32>().ok()?;

    // Basic validation.
    if month == 0 || month > 12 || day == 0 || day > 31 {
        return None;
    }

    Some((year, month, day))
}

/// Check if a date is before the expiry cutoff (2026-02-27).
const fn is_expired(year: u32, month: u32, day: u32) -> bool {
    if year < EXPIRY_YEAR {
        return true;
    }
    if year == EXPIRY_YEAR {
        if month < EXPIRY_MONTH {
            return true;
        }
        if month == EXPIRY_MONTH && day < EXPIRY_DAY {
            return true;
        }
    }
    false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExpiringTodoComments)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_expired_todo_bracket() {
        let diags = lint("// TODO [2025-01-01]: fix this");
        assert_eq!(
            diags.len(),
            1,
            "expired TODO with bracket date should be flagged"
        );
    }

    #[test]
    fn test_flags_expired_fixme_bracket() {
        let diags = lint("// FIXME [2024-06-15]: remove hack");
        assert_eq!(
            diags.len(),
            1,
            "expired FIXME with bracket date should be flagged"
        );
    }

    #[test]
    fn test_flags_expired_todo_paren() {
        let diags = lint("// TODO (2023-12-31): old task");
        assert_eq!(
            diags.len(),
            1,
            "expired TODO with paren date should be flagged"
        );
    }

    #[test]
    fn test_flags_expired_multiline_comment() {
        let diags = lint("/* TODO [2025-06-01]: clean up */");
        assert_eq!(
            diags.len(),
            1,
            "expired TODO in multi-line comment should be flagged"
        );
    }

    #[test]
    fn test_allows_future_todo() {
        let diags = lint("// TODO [2027-01-01]: future work");
        assert!(diags.is_empty(), "future TODO date should not be flagged");
    }

    #[test]
    fn test_allows_exact_cutoff() {
        let diags = lint("// TODO [2026-02-27]: on the cutoff");
        assert!(
            diags.is_empty(),
            "TODO on exact cutoff date should not be flagged"
        );
    }

    #[test]
    fn test_allows_todo_without_date() {
        let diags = lint("// TODO: no date here");
        assert!(
            diags.is_empty(),
            "TODO without a date should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_comment() {
        let diags = lint("// Regular comment about the code");
        assert!(diags.is_empty(), "regular comment should not be flagged");
    }

    #[test]
    fn test_allows_date_in_string() {
        let diags = lint("var x = 'TODO [2020-01-01]: in a string';");
        assert!(
            diags.is_empty(),
            "TODO in string literal should not be flagged"
        );
    }

    #[test]
    fn test_flags_lowercase_todo() {
        let diags = lint("// todo [2025-03-15]: lowercase");
        assert_eq!(
            diags.len(),
            1,
            "lowercase todo with expired date should be flagged"
        );
    }

    #[test]
    fn test_allows_future_year() {
        let diags = lint("// TODO 2028-06-15: far future");
        assert!(
            diags.is_empty(),
            "TODO with a far future date should not be flagged"
        );
    }
}
