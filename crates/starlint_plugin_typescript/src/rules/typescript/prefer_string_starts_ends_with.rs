//! Rule: `typescript/prefer-string-starts-ends-with`
//!
//! Prefer `String.startsWith()` and `String.endsWith()` over equivalent
//! string methods. Patterns like `.charAt(0) === 'x'`, `.indexOf(x) === 0`,
//! `.slice(0, n) === '...'`, and `.substring(0, n) === '...'` can all be
//! replaced with the more readable `.startsWith()` / `.endsWith()`.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! Flagged patterns:
//! - `.charAt(0) ===` / `.charAt(0) ==`
//! - `.indexOf(x) === 0` / `.indexOf(x) == 0`
//! - `.slice(0,` followed by `) ===` / `) ==`
//! - `.substring(0,` followed by `) ===` / `) ==`

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags string method patterns that can be replaced with `startsWith()` / `endsWith()`.
#[derive(Debug)]
pub struct PreferStringStartsEndsWith;

impl LintRule for PreferStringStartsEndsWith {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-string-starts-ends-with".to_owned(),
            description: "Prefer `startsWith()` / `endsWith()` over equivalent string methods"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains(".indexOf(")
            || source_text.contains(".charAt(")
            || source_text.contains("[0]")
            || source_text.contains(".slice(")
            || source_text.contains(".substring(")
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_starts_ends_with_patterns(source);

        for (msg, start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/prefer-string-starts-ends-with".to_owned(),
                message: msg.to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Use `startsWith()` or `endsWith()` instead".to_owned()),
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Scan source text for patterns that can be replaced with `startsWith()` / `endsWith()`.
///
/// Returns a list of `(message, start_offset, end_offset)` tuples.
fn find_starts_ends_with_patterns(source: &str) -> Vec<(&'static str, u32, u32)> {
    let mut results = Vec::new();

    find_charat_zero_patterns(source, &mut results);
    find_indexof_zero_patterns(source, &mut results);
    find_slice_zero_patterns(source, &mut results);
    find_substring_zero_patterns(source, &mut results);

    results
}

/// Detect `.charAt(0) ===` and `.charAt(0) ==` patterns.
fn find_charat_zero_patterns(source: &str, results: &mut Vec<(&'static str, u32, u32)>) {
    let needle = ".charAt(0)";
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(needle)) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_needle = absolute_pos.saturating_add(needle.len());
        let rest = source.get(after_needle..).unwrap_or("");
        let trimmed = rest.trim_start();

        if trimmed.starts_with("===")
            || trimmed.starts_with("==")
            || trimmed.starts_with("!==")
            || trimmed.starts_with("!=")
        {
            let start = u32::try_from(absolute_pos).unwrap_or(0);
            let end = u32::try_from(after_needle).unwrap_or(start);
            results.push((
                "Use `startsWith()` instead of `.charAt(0)` comparison",
                start,
                end,
            ));
        }

        search_from = after_needle;
    }
}

/// Detect `.indexOf(x) === 0` and `.indexOf(x) == 0` patterns.
fn find_indexof_zero_patterns(source: &str, results: &mut Vec<(&'static str, u32, u32)>) {
    let needle = ".indexOf(";
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(needle)) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_needle = absolute_pos.saturating_add(needle.len());

        if let Some(close_paren) = find_closing_paren(source, after_needle) {
            let after_close = close_paren.saturating_add(1);
            let rest = source.get(after_close..).unwrap_or("");
            let trimmed = rest.trim_start();

            if trimmed.starts_with("=== 0") || trimmed.starts_with("== 0") {
                let start = u32::try_from(absolute_pos).unwrap_or(0);
                let end = u32::try_from(after_close).unwrap_or(start);
                results.push((
                    "Use `startsWith()` instead of `.indexOf(x) === 0`",
                    start,
                    end,
                ));
            }
        }

        search_from = after_needle;
    }
}

/// Detect `.slice(0, n) ===` patterns.
fn find_slice_zero_patterns(source: &str, results: &mut Vec<(&'static str, u32, u32)>) {
    find_prefix_method_pattern(source, ".slice(0,", results);
}

/// Detect `.substring(0, n) ===` patterns.
fn find_substring_zero_patterns(source: &str, results: &mut Vec<(&'static str, u32, u32)>) {
    find_prefix_method_pattern(source, ".substring(0,", results);
}

/// Generic helper to detect `.method(0, n) ===` patterns.
fn find_prefix_method_pattern(
    source: &str,
    needle: &str,
    results: &mut Vec<(&'static str, u32, u32)>,
) {
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(needle)) {
        let absolute_pos = search_from.saturating_add(pos);

        // Find closing paren starting from after the opening `(`
        let open_paren_pos =
            absolute_pos.saturating_add(needle.find('(').unwrap_or(needle.len()).saturating_add(1));
        if let Some(close_paren) = find_closing_paren(source, open_paren_pos) {
            let after_close = close_paren.saturating_add(1);
            let rest = source.get(after_close..).unwrap_or("");
            let trimmed = rest.trim_start();

            if trimmed.starts_with("===")
                || trimmed.starts_with("==")
                || trimmed.starts_with("!==")
                || trimmed.starts_with("!=")
            {
                let start = u32::try_from(absolute_pos).unwrap_or(0);
                let end = u32::try_from(after_close).unwrap_or(start);
                results.push((
                    "Use `startsWith()` instead of string prefix comparison",
                    start,
                    end,
                ));
            }
        }

        search_from = absolute_pos.saturating_add(needle.len());
    }
}

/// Find the matching closing parenthesis, handling nesting.
///
/// `start` is the position right after the opening `(`.
fn find_closing_paren(source: &str, start: usize) -> Option<usize> {
    let mut depth: u32 = 1;
    let mut pos = start;
    let bytes = source.as_bytes();
    let len = bytes.len();

    while pos < len {
        match bytes.get(pos).copied() {
            Some(b'(') => depth = depth.saturating_add(1),
            Some(b')') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
        pos = pos.saturating_add(1);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(PreferStringStartsEndsWith, "test.ts");

    #[test]
    fn test_flags_charat_zero_comparison() {
        let diags = lint("if (str.charAt(0) === 'a') {}");
        assert_eq!(diags.len(), 1, ".charAt(0) comparison should be flagged");
    }

    #[test]
    fn test_flags_indexof_equal_zero() {
        let diags = lint("if (str.indexOf('abc') === 0) {}");
        assert_eq!(diags.len(), 1, ".indexOf() === 0 should be flagged");
    }

    #[test]
    fn test_flags_slice_zero_comparison() {
        let diags = lint("if (str.slice(0, 3) === 'abc') {}");
        assert_eq!(diags.len(), 1, ".slice(0, n) comparison should be flagged");
    }

    #[test]
    fn test_flags_substring_zero_comparison() {
        let diags = lint("if (str.substring(0, 3) === 'abc') {}");
        assert_eq!(
            diags.len(),
            1,
            ".substring(0, n) comparison should be flagged"
        );
    }

    #[test]
    fn test_allows_includes_call() {
        let diags = lint("if (str.startsWith('abc')) {}");
        assert!(diags.is_empty(), ".startsWith() should not be flagged");
    }
}
