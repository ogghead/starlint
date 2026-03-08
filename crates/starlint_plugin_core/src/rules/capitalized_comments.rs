//! Rule: `capitalized-comments` (eslint)
//!
//! Flag comments that start with a lowercase letter. Comments should begin
//! with a capital letter for consistency and readability.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Pragma prefixes that are exempt from capitalization checks.
const PRAGMAS: &[&str] = &[
    "todo",
    "fixme",
    "hack",
    "xxx",
    "eslint",
    "jshint",
    "jslint",
    "istanbul",
    "global",
    "globals",
    "exported",
    "jscs",
    "falls through",
    "c8",
    "v8",
    "type:",
    "ts-",
    "prettier-",
];

/// Scan source text for comments that start with a lowercase letter.
///
/// Returns a list of `(comment_span, first_char_offset)` for each offending comment.
fn find_bad_comments(source: &str) -> Vec<(Span, usize)> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos: usize = 0;
    let mut results = Vec::new();

    while pos < len {
        // Check for line comments
        let remaining = source.get(pos..).unwrap_or("");
        if remaining.starts_with("//") {
            let comment_start = pos;
            let content_start = pos.saturating_add(2);
            let line_end = source
                .get(content_start..)
                .and_then(|s| s.find('\n'))
                .map_or(len, |offset| content_start.saturating_add(offset));
            let comment_text = source.get(content_start..line_end).unwrap_or("");

            if check_comment_text(comment_text) {
                let span_start = u32::try_from(comment_start).unwrap_or(0);
                let span_end = u32::try_from(line_end).unwrap_or(0);
                let first_alpha_offset = content_start.saturating_add(
                    comment_text
                        .len()
                        .saturating_sub(comment_text.trim_start().len()),
                );
                results.push((Span::new(span_start, span_end), first_alpha_offset));
            }
            pos = line_end;
            continue;
        }

        // Check for block comments
        if remaining.starts_with("/*") {
            let comment_start = pos;
            let content_start = pos.saturating_add(2);
            let block_end = source
                .get(content_start..)
                .and_then(|s| s.find("*/"))
                .map_or(len, |offset| {
                    content_start.saturating_add(offset).saturating_add(2)
                });
            let content_end = block_end.saturating_sub(2);
            let comment_text = source.get(content_start..content_end).unwrap_or("");

            if check_comment_text(comment_text) {
                let span_start = u32::try_from(comment_start).unwrap_or(0);
                let span_end = u32::try_from(block_end).unwrap_or(0);
                let first_alpha_offset = content_start.saturating_add(
                    comment_text
                        .len()
                        .saturating_sub(comment_text.trim_start().len()),
                );
                results.push((Span::new(span_start, span_end), first_alpha_offset));
            }
            pos = block_end;
            continue;
        }

        // Skip string literals to avoid false positives
        let current_byte = bytes.get(pos).copied().unwrap_or(0);
        if current_byte == b'"' || current_byte == b'\'' || current_byte == b'`' {
            let quote = current_byte;
            pos = pos.saturating_add(1);
            while pos < len {
                let ch = bytes.get(pos).copied().unwrap_or(0);
                if ch == b'\\' {
                    // Skip escaped character
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

/// Flags comments that start with a lowercase letter.
#[derive(Debug)]
pub struct CapitalizedComments;

/// Check whether a comment's text content (after stripping the comment marker
/// and leading whitespace) starts with a lowercase letter.
///
/// Returns `Some((byte_offset_in_source, length))` if the comment should be
/// flagged, or `None` if it's fine.
fn check_comment_text(text: &str) -> bool {
    let trimmed = text.trim_start();

    // Skip empty comments
    if trimmed.is_empty() {
        return false;
    }

    // The first non-whitespace character must be alphabetic to be checked
    let first_char = trimmed.chars().next();
    let Some(ch) = first_char else {
        return false;
    };

    // If the comment doesn't start with a letter, skip it
    // (e.g. `// 123`, `// ---`, `// @param`)
    if !ch.is_alphabetic() {
        return false;
    }

    // Skip pragma comments
    let lower_trimmed = trimmed.to_lowercase();
    for pragma in PRAGMAS {
        if lower_trimmed.starts_with(pragma) {
            return false;
        }
    }

    // Skip URLs
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return false;
    }

    // Flag if the first character is lowercase
    ch.is_lowercase()
}

impl LintRule for CapitalizedComments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "capitalized-comments".to_owned(),
            description: "Require comments to begin with a capital letter".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("//") || source_text.contains("/*")
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();
        let flagged = find_bad_comments(&source);

        for (span, first_alpha_offset) in flagged {
            // Build fix: capitalize the first letter
            let fix = source.get(first_alpha_offset..).and_then(|s| {
                let ch = s.chars().next()?;
                let upper: String = ch.to_uppercase().collect();
                let char_end = first_alpha_offset.saturating_add(ch.len_utf8());
                let fix_start = u32::try_from(first_alpha_offset).ok()?;
                let fix_end = u32::try_from(char_end).ok()?;
                Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Capitalize `{ch}` to `{upper}`"),
                    edits: vec![Edit {
                        span: Span::new(fix_start, fix_end),
                        replacement: upper,
                    }],
                    is_snippet: false,
                })
            });

            ctx.report(Diagnostic {
                rule_name: "capitalized-comments".to_owned(),
                message: "Comments should start with an uppercase letter".to_owned(),
                span,
                severity: Severity::Warning,
                help: Some("Capitalize the first letter of the comment".to_owned()),
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CapitalizedComments)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_lowercase_line_comment() {
        let diags = lint("// this is bad");
        assert_eq!(diags.len(), 1, "lowercase line comment should be flagged");
    }

    #[test]
    fn test_allows_uppercase_line_comment() {
        let diags = lint("// This is good");
        assert!(
            diags.is_empty(),
            "uppercase line comment should not be flagged"
        );
    }

    #[test]
    fn test_allows_todo_pragma() {
        let diags = lint("// TODO: fix this");
        assert!(diags.is_empty(), "TODO pragma should not be flagged");
    }

    #[test]
    fn test_allows_fixme_pragma() {
        let diags = lint("// fixme: something");
        assert!(diags.is_empty(), "fixme pragma should not be flagged");
    }

    #[test]
    fn test_flags_lowercase_block_comment() {
        let diags = lint("/* this is bad */");
        assert_eq!(diags.len(), 1, "lowercase block comment should be flagged");
    }

    #[test]
    fn test_allows_uppercase_block_comment() {
        let diags = lint("/* This is good */");
        assert!(
            diags.is_empty(),
            "uppercase block comment should not be flagged"
        );
    }

    #[test]
    fn test_allows_numeric_comment() {
        let diags = lint("// 123 numeric");
        assert!(
            diags.is_empty(),
            "comment starting with number should not be flagged"
        );
    }

    #[test]
    fn test_allows_eslint_disable() {
        let diags = lint("// eslint-disable-next-line no-console");
        assert!(diags.is_empty(), "eslint directive should not be flagged");
    }
}
