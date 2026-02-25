//! Diagnostic formatting for output.
//!
//! Supports pretty (human-readable), JSON, and compact output formats.

use std::fmt::Write;
use std::path::Path;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity};

/// Output format for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable colored output.
    #[default]
    Pretty,
    /// JSON output (one object per diagnostic).
    Json,
    /// Compact single-line format.
    Compact,
}

/// Format a collection of diagnostics for a single file.
#[must_use]
pub fn format_diagnostics(
    diagnostics: &[Diagnostic],
    source_text: &str,
    file_path: &Path,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Pretty => format_pretty(diagnostics, source_text, file_path),
        OutputFormat::Json => format_json(diagnostics, file_path),
        OutputFormat::Compact => format_compact(diagnostics, file_path),
    }
}

/// Format diagnostics in human-readable form.
#[allow(clippy::let_underscore_must_use)] // writeln! to String is infallible
fn format_pretty(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = String::new();

    for diag in diagnostics {
        let (line, col) = offset_to_line_col(source_text, diag.span.start);
        let severity_str = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Suggestion => "suggestion",
        };

        let _ = writeln!(
            output,
            "  {severity_str}[{rule}]: {message}",
            rule = diag.rule_name,
            message = diag.message,
        );
        let _ = writeln!(
            output,
            "    --> {path}:{line}:{col}",
            path = file_path.display(),
        );

        if let Some(help) = &diag.help {
            let _ = writeln!(output, "    help: {help}");
        }

        output.push('\n');
    }

    output
}

/// Format diagnostics as newline-delimited JSON (NDJSON).
///
/// Each diagnostic is emitted as a standalone JSON object on its own line,
/// rather than wrapped in a JSON array. This is compatible with tools like
/// `jq` and line-oriented log processors.
fn format_json(diagnostics: &[Diagnostic], file_path: &Path) -> String {
    let mut entries = Vec::new();
    for diag in diagnostics {
        let entry = serde_json::json!({
            "file": file_path.display().to_string(),
            "rule": diag.rule_name,
            "message": diag.message,
            "severity": diag.severity,
            "span": { "start": diag.span.start, "end": diag.span.end },
            "help": diag.help,
        });
        match serde_json::to_string(&entry) {
            Ok(json_str) => entries.push(json_str),
            Err(err) => {
                tracing::warn!(
                    "failed to serialize diagnostic for rule '{}': {err}",
                    diag.rule_name
                );
            }
        }
    }
    entries.join("\n")
}

/// Format diagnostics in compact single-line form.
#[allow(clippy::let_underscore_must_use)] // writeln! to String is infallible
fn format_compact(diagnostics: &[Diagnostic], file_path: &Path) -> String {
    let mut output = String::new();
    for diag in diagnostics {
        let severity_char = match diag.severity {
            Severity::Error => 'E',
            Severity::Warning => 'W',
            Severity::Suggestion => 'S',
        };
        let _ = writeln!(
            output,
            "{path}:{start}-{end} {sev} [{rule}] {message}",
            path = file_path.display(),
            start = diag.span.start,
            end = diag.span.end,
            sev = severity_char,
            rule = diag.rule_name,
            message = diag.message,
        );
    }
    output
}

/// Convert a byte offset to 1-based line and column numbers.
fn offset_to_line_col(source: &str, offset: u32) -> (usize, usize) {
    let mut line: usize = 1;
    let mut col: usize = 1;
    let offset_usize: usize = offset.try_into().unwrap_or(usize::MAX);
    for (i, ch) in source.char_indices() {
        if i >= offset_usize {
            break;
        }
        if ch == '\n' {
            line = line.saturating_add(1);
            col = 1;
        } else {
            col = col.saturating_add(1);
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use starlint_plugin_sdk::diagnostic::Span;

    fn make_diag(rule: &str, message: &str, severity: Severity) -> Diagnostic {
        Diagnostic {
            rule_name: rule.to_owned(),
            message: message.to_owned(),
            span: Span::new(0, 5),
            severity,
            help: None,
            fix: None,
            labels: vec![],
        }
    }

    #[test]
    fn test_offset_to_line_col() {
        let source = "abc\ndef\nghi";
        assert_eq!(offset_to_line_col(source, 0), (1, 1), "start of file");
        assert_eq!(
            offset_to_line_col(source, 4),
            (2, 1),
            "start of second line"
        );
        assert_eq!(
            offset_to_line_col(source, 5),
            (2, 2),
            "second char of second line"
        );
    }

    #[test]
    fn test_offset_to_line_col_multibyte() {
        // 'ä' is 2 bytes in UTF-8.
        let source = "ä\nb";
        assert_eq!(offset_to_line_col(source, 0), (1, 1), "start of file");
        assert_eq!(offset_to_line_col(source, 2), (1, 2), "after ä");
        assert_eq!(
            offset_to_line_col(source, 3),
            (2, 1),
            "start of second line"
        );
    }

    #[test]
    fn test_offset_to_line_col_emoji() {
        // '😀' is 4 bytes in UTF-8.
        let source = "a😀b";
        assert_eq!(offset_to_line_col(source, 0), (1, 1), "'a'");
        assert_eq!(offset_to_line_col(source, 1), (1, 2), "start of emoji");
        assert_eq!(offset_to_line_col(source, 5), (1, 3), "'b' after emoji");
    }

    #[test]
    fn test_format_compact() {
        let diag = make_diag("no-debugger", "bad", Severity::Error);
        let output = format_compact(&[diag], Path::new("test.js"));
        assert!(
            output.contains("test.js:0-5 E [no-debugger] bad"),
            "compact format should contain expected fields: {output}"
        );
    }

    #[test]
    fn test_format_pretty() {
        let diag = make_diag("no-debugger", "bad code", Severity::Warning);
        let output = format_pretty(&[diag], "debugger;", Path::new("test.js"));
        assert!(
            output.contains("warning[no-debugger]"),
            "should contain severity and rule"
        );
        assert!(
            output.contains("test.js:1:1"),
            "should contain file location"
        );
    }

    #[test]
    fn test_format_json() {
        let diag = make_diag("no-debugger", "bad", Severity::Error);
        let output = format_json(&[diag], Path::new("test.js"));
        assert!(
            output.contains("\"rule\":\"no-debugger\""),
            "json should contain rule name"
        );
    }
}
