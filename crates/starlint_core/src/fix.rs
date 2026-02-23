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
    let mut fixes: Vec<&Fix> = diagnostics.iter().filter_map(|d| d.fix.as_ref()).collect();

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

            if start <= result.len() && end <= result.len() {
                result.replace_range(start..end, &edit.replacement);
                last_edit_start = edit.span.start;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use starlint_plugin_sdk::diagnostic::{Edit, Severity, Span};

    fn make_diag_with_fix(span: Span, replacement: &str) -> Diagnostic {
        Diagnostic {
            rule_name: "test".to_owned(),
            message: "test".to_owned(),
            span,
            severity: Severity::Error,
            help: None,
            fix: Some(Fix {
                message: "fix it".to_owned(),
                edits: vec![Edit {
                    span,
                    replacement: replacement.to_owned(),
                }],
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
}
