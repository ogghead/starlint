//! Auto-fix application.
//!
//! Applies text edits from diagnostics to source text, producing a fixed version.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix};

/// Apply all safe fixes from diagnostics to source text.
///
/// Returns the modified source text with fixes applied.
/// Fixes are applied in reverse order by span start to preserve offsets.
/// Overlapping fixes are skipped.
#[must_use]
pub fn apply_fixes(source: &str, diagnostics: &[Diagnostic]) -> String {
    let mut fixes: Vec<&Fix> = diagnostics
        .iter()
        .filter_map(|d| d.fix.as_ref())
        .filter(|f| !f.is_snippet)
        .collect();

    if fixes.is_empty() {
        return source.to_owned();
    }

    // Sort fixes by first edit's span start, descending (apply from end to start).
    fixes.sort_by(|a, b| {
        let a_start = a.edits.first().map_or(0, |e| e.span.start);
        let b_start = b.edits.first().map_or(0, |e| e.span.start);
        b_start.cmp(&a_start)
    });

    let mut result = source.to_owned();
    let mut last_edit_start = u32::MAX;

    for fix in &fixes {
        let mut edits = fix.edits.clone();
        // Sort edits within a fix descending by span start.
        edits.sort_by(|a, b| b.span.start.cmp(&a.span.start));

        for edit in &edits {
            let start: usize = edit.span.start.try_into().unwrap_or(0);
            let end: usize = edit.span.end.try_into().unwrap_or(0);

            // Skip overlapping edits.
            if edit.span.end > last_edit_start {
                continue;
            }

            // Guard: skip invalid spans (inverted, out-of-bounds, or mid-UTF-8).
            if start > end
                || end > result.len()
                || !result.is_char_boundary(start)
                || !result.is_char_boundary(end)
            {
                continue;
            }

            result.replace_range(start..end, &edit.replacement);
            last_edit_start = edit.span.start;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use starlint_plugin_sdk::diagnostic::{Edit, Severity, Span};
    use starlint_plugin_sdk::rule::FixKind;

    fn make_diag_with_fix(span: Span, replacement: &str) -> Diagnostic {
        Diagnostic {
            rule_name: "test".to_owned(),
            message: "test".to_owned(),
            span,
            severity: Severity::Error,
            help: None,
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "fix it".to_owned(),
                edits: vec![Edit {
                    span,
                    replacement: replacement.to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        }
    }

    #[test]
    fn test_apply_single_fix() {
        let source = "debugger;\nconst x = 1;";
        let diag = make_diag_with_fix(Span::new(0, 10), "");
        let result = apply_fixes(source, &[diag]);
        assert_eq!(result, "const x = 1;", "debugger line should be removed");
    }

    #[test]
    fn test_apply_no_fixes() {
        let source = "const x = 1;";
        let diag = Diagnostic {
            rule_name: "test".to_owned(),
            message: "no fix".to_owned(),
            span: Span::new(0, 5),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        };
        let result = apply_fixes(source, &[diag]);
        assert_eq!(result, source, "source should be unchanged without fixes");
    }

    #[test]
    fn test_apply_multiple_non_overlapping_fixes() {
        let source = "aaa bbb ccc";
        let diags = vec![
            make_diag_with_fix(Span::new(0, 3), "xxx"),
            make_diag_with_fix(Span::new(8, 11), "zzz"),
        ];
        let result = apply_fixes(source, &diags);
        assert_eq!(result, "xxx bbb zzz", "both fixes should apply");
    }

    #[test]
    fn test_inverted_span_does_not_panic() {
        let source = "hello world";
        let diag = make_diag_with_fix(Span::new(5, 3), "X");
        let result = apply_fixes(source, &[diag]);
        assert_eq!(
            result, source,
            "inverted span (start > end) should be skipped"
        );
    }

    #[test]
    fn test_mid_utf8_span_does_not_panic() {
        // 'ä' is 2 bytes (0xC3 0xA4). Offset 1 falls mid-character.
        let source = "ä";
        let diag = make_diag_with_fix(Span::new(1, 2), "X");
        let result = apply_fixes(source, &[diag]);
        assert_eq!(result, source, "mid-UTF-8 byte offset should be skipped");
    }

    #[test]
    fn test_snippet_fixes_are_skipped() {
        let source = "aaa bbb";
        let diag = Diagnostic {
            rule_name: "test".to_owned(),
            message: "test".to_owned(),
            span: Span::new(0, 3),
            severity: Severity::Warning,
            help: None,
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "snippet fix".to_owned(),
                edits: vec![Edit {
                    span: Span::new(0, 3),
                    replacement: "${1:xxx}".to_owned(),
                }],
                is_snippet: true,
            }),
            labels: vec![],
        };
        let result = apply_fixes(source, &[diag]);
        assert_eq!(result, source, "snippet fixes should be skipped by CLI");
    }

    #[test]
    fn test_overlapping_fixes_skip_second() {
        let source = "abcdefgh";
        let diags = vec![
            make_diag_with_fix(Span::new(2, 6), "XX"),
            make_diag_with_fix(Span::new(4, 8), "YY"),
        ];
        let result = apply_fixes(source, &diags);
        // Fixes sorted descending by start: (4,8) applied first, then (2,6) overlaps → skipped.
        assert_eq!(result, "abcdYY", "overlapping fix should be skipped");
    }
}
