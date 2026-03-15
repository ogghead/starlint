//! Rule: `typescript/prefer-regexp-exec`
//!
//! Prefer `RegExp.exec()` over `String.match()` when not using the global
//! flag. `String.match()` and `RegExp.exec()` behave identically when the
//! regex does not have the `g` flag, but `RegExp.exec()` is more performant
//! and communicates intent more clearly.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! Flagged patterns:
//! - `.match(/regex/)` where the regex does not include the `g` flag
//!
//! Allowed patterns:
//! - `.match(/regex/g)` — global matching has different semantics
//! - `.match(/regex/gi)` — any combination containing `g`

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.match(/regex/)` calls where `RegExp.exec()` would be preferred.
#[derive(Debug)]
pub struct PreferRegexpExec;

impl LintRule for PreferRegexpExec {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-regexp-exec".to_owned(),
            description: "Prefer `RegExp.exec()` over `String.match()` without global flag"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_match_without_global(source);

        for (start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/prefer-regexp-exec".to_owned(),
                message:
                    "Use `RegExp.exec()` instead of `String.match()` when not using the global flag"
                        .to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// The pattern prefix to search for: `.match(/`.
const MATCH_REGEX_PREFIX: &str = ".match(/";

/// Scan source text for `.match(/regex/)` patterns where the regex does not
/// contain the `g` flag.
///
/// Returns a list of `(start_offset, end_offset)` for each occurrence.
fn find_match_without_global(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source
        .get(search_from..)
        .and_then(|s| s.find(MATCH_REGEX_PREFIX))
    {
        let absolute_pos = search_from.saturating_add(pos);
        // Position of the `/` that starts the regex
        let regex_start = absolute_pos.saturating_add(MATCH_REGEX_PREFIX.len());

        if let Some((regex_end, flags)) = find_regex_end_and_flags(source, regex_start) {
            // Only flag if the `g` flag is NOT present
            if !flags.contains('g') {
                // Check that the regex call ends with `)`
                let after_flags = regex_end.saturating_add(flags.len());
                let rest = source.get(after_flags..).unwrap_or("");
                let trimmed = rest.trim_start();

                if trimmed.starts_with(')') {
                    let close_paren = after_flags
                        .saturating_add(rest.len().saturating_sub(trimmed.len()))
                        .saturating_add(1);
                    let start = u32::try_from(absolute_pos).unwrap_or(0);
                    let end = u32::try_from(close_paren).unwrap_or(start);
                    results.push((start, end));
                }
            }
        }

        search_from = regex_start;
    }

    results
}

/// Starting at the character after the opening `/` of a regex literal,
/// find the closing `/` and extract the flags that follow.
///
/// Handles escaped characters (`\/`) inside the regex body.
///
/// Returns `(position_after_closing_slash, flags_string)`.
fn find_regex_end_and_flags(source: &str, start: usize) -> Option<(usize, &str)> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos = start;
    let mut escaped = false;

    while pos < len {
        let byte = bytes.get(pos).copied().unwrap_or(0);

        if escaped {
            escaped = false;
            pos = pos.saturating_add(1);
            continue;
        }

        if byte == b'\\' {
            escaped = true;
            pos = pos.saturating_add(1);
            continue;
        }

        if byte == b'\n' || byte == b'\r' {
            // Regex literals cannot span lines
            return None;
        }

        if byte == b'/' {
            // Found the closing slash
            let after_slash = pos.saturating_add(1);

            // Collect flags (alphabetic characters immediately after `/`)
            let mut flag_end = after_slash;
            while flag_end < len {
                match bytes.get(flag_end).copied() {
                    Some(b) if b.is_ascii_alphabetic() => {
                        flag_end = flag_end.saturating_add(1);
                    }
                    _ => break,
                }
            }

            let flags = source.get(after_slash..flag_end).unwrap_or("");
            return Some((after_slash, flags));
        }

        pos = pos.saturating_add(1);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(PreferRegexpExec, "test.ts");

    #[test]
    fn test_flags_match_without_global() {
        let diags = lint("const m = str.match(/foo/);");
        assert_eq!(diags.len(), 1, ".match() without g flag should be flagged");
    }

    #[test]
    fn test_flags_match_with_case_insensitive_only() {
        let diags = lint("const m = str.match(/foo/i);");
        assert_eq!(
            diags.len(),
            1,
            ".match() with only i flag (no g) should be flagged"
        );
    }

    #[test]
    fn test_allows_match_with_global_flag() {
        let diags = lint("const m = str.match(/foo/g);");
        assert!(
            diags.is_empty(),
            ".match() with g flag should not be flagged"
        );
    }

    #[test]
    fn test_allows_match_with_global_and_insensitive() {
        let diags = lint("const m = str.match(/foo/gi);");
        assert!(
            diags.is_empty(),
            ".match() with gi flags should not be flagged"
        );
    }

    #[test]
    fn test_allows_exec_call() {
        let diags = lint("const m = /foo/.exec(str);");
        assert!(diags.is_empty(), "RegExp.exec() should not be flagged");
    }
}
