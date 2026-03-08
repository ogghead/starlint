//! Rule: `typescript/switch-exhaustiveness-check`
//!
//! Require switch statements to be exhaustive. Without full type information,
//! this simplified version flags `switch` statements that do not include a
//! `default` case. A missing `default` case can lead to silent bugs when a
//! new variant is added to the discriminant type.
//!
//! Simplified syntax-only version — full checking requires type information.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/switch-exhaustiveness-check";

/// Flags `switch` statements that do not contain a `default` case.
#[derive(Debug)]
pub struct SwitchExhaustivenessCheck;

impl LintRule for SwitchExhaustivenessCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require switch statements to have a `default` case for exhaustiveness"
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
        let violations = find_switch_without_default(source);

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Switch statement is missing a `default` case — add one to ensure exhaustiveness".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Scan source text for `switch` statements that lack a `default:` case.
///
/// This text-based heuristic finds `switch` keywords, tracks brace depth to
/// locate the matching closing brace, then checks if `default` (followed by
/// `:`) appears within that span.
fn find_switch_without_default(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let needle = b"switch";
    let needle_len = needle.len();
    let mut search_from: usize = 0;

    while search_from.saturating_add(needle_len) <= len {
        // Find next occurrence of "switch"
        let Some(pos) = find_bytes(bytes, needle, search_from) else {
            break;
        };

        // Verify it's a standalone keyword (not part of a larger identifier)
        if !is_word_boundary(bytes, pos, needle_len) {
            search_from = pos.saturating_add(1);
            continue;
        }

        // Find the opening brace for this switch
        let Some(open_brace) = find_char(bytes, b'{', pos.saturating_add(needle_len)) else {
            search_from = pos.saturating_add(1);
            continue;
        };

        // Find the matching closing brace
        let Some(close_brace) = find_matching_brace(bytes, open_brace) else {
            search_from = pos.saturating_add(1);
            continue;
        };

        // Extract the body between braces and check for `default:`
        let body_start = open_brace.saturating_add(1);
        let body_end = close_brace;

        if !has_default_case(source, body_start, body_end) {
            let start = u32::try_from(pos).unwrap_or(0);
            let end = u32::try_from(close_brace.saturating_add(1)).unwrap_or(start);
            results.push(Span::new(start, end));
        }

        search_from = close_brace.saturating_add(1);
    }

    results
}

/// Find a byte sequence in a byte slice starting from an offset.
fn find_bytes(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    let search_area = haystack.get(from..)?;
    let needle_len = needle.len();

    for i in 0..search_area
        .len()
        .saturating_sub(needle_len.saturating_sub(1))
    {
        if search_area.get(i..i.saturating_add(needle_len)) == Some(needle) {
            return Some(from.saturating_add(i));
        }
    }

    None
}

/// Check that the position marks a word boundary (not inside a larger identifier).
fn is_word_boundary(bytes: &[u8], pos: usize, keyword_len: usize) -> bool {
    // Character before must not be alphanumeric or underscore
    if pos > 0 {
        let before = bytes.get(pos.saturating_sub(1)).copied().unwrap_or(b' ');
        if before.is_ascii_alphanumeric() || before == b'_' || before == b'$' {
            return false;
        }
    }

    // Character after must not be alphanumeric or underscore
    let after_pos = pos.saturating_add(keyword_len);
    let after = bytes.get(after_pos).copied().unwrap_or(b' ');
    if after.is_ascii_alphanumeric() || after == b'_' || after == b'$' {
        return false;
    }

    true
}

/// Find the first occurrence of a character starting from an offset.
fn find_char(bytes: &[u8], ch: u8, from: usize) -> Option<usize> {
    for (i, byte) in bytes.get(from..)?.iter().enumerate() {
        if *byte == ch {
            return Some(from.saturating_add(i));
        }
    }
    None
}

/// Find the matching closing brace, tracking nested brace depth.
fn find_matching_brace(bytes: &[u8], open_pos: usize) -> Option<usize> {
    let mut depth: u32 = 1;
    let mut pos = open_pos.saturating_add(1);

    while pos < bytes.len() {
        match bytes.get(pos).copied().unwrap_or(0) {
            b'{' => {
                depth = depth.saturating_add(1);
            }
            b'}' => {
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

/// Check if the switch body contains a `default:` case label.
fn has_default_case(source: &str, body_start: usize, body_end: usize) -> bool {
    let body = source.get(body_start..body_end).unwrap_or("");
    let needle = "default";
    let needle_len = needle.len();

    let mut search_from: usize = 0;
    while let Some(pos) = body.get(search_from..).and_then(|s| s.find(needle)) {
        let abs_pos = search_from.saturating_add(pos);

        // Check word boundary
        if abs_pos > 0 {
            let before = body
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .copied()
                .unwrap_or(b' ');
            if before.is_ascii_alphanumeric() || before == b'_' {
                search_from = abs_pos.saturating_add(1);
                continue;
            }
        }

        // Check that it's followed (possibly with whitespace) by a colon
        let after_default = body.get(abs_pos.saturating_add(needle_len)..).unwrap_or("");
        let trimmed = after_default.trim_start();
        if trimmed.starts_with(':') {
            return true;
        }

        search_from = abs_pos.saturating_add(1);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(SwitchExhaustivenessCheck)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_switch_without_default() {
        let diags = lint("switch (x) { case 1: break; case 2: break; }");
        assert_eq!(
            diags.len(),
            1,
            "switch without default case should be flagged"
        );
    }

    #[test]
    fn test_allows_switch_with_default() {
        let diags = lint("switch (x) { case 1: break; default: break; }");
        assert!(
            diags.is_empty(),
            "switch with default case should not be flagged"
        );
    }

    #[test]
    fn test_allows_default_with_whitespace_before_colon() {
        let diags = lint("switch (x) { case 1: break; default : break; }");
        assert!(
            diags.is_empty(),
            "default with space before colon should not be flagged"
        );
    }

    #[test]
    fn test_flags_empty_switch() {
        let diags = lint("switch (x) { case 1: break; }");
        assert_eq!(
            diags.len(),
            1,
            "switch with only one case and no default should be flagged"
        );
    }

    #[test]
    fn test_does_not_flag_default_in_identifier() {
        let diags = lint("switch (x) { case 1: defaultValue(); break; }");
        assert_eq!(
            diags.len(),
            1,
            "defaultValue identifier should not count as a default case"
        );
    }
}
