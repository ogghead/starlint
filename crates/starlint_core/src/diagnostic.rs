//! Diagnostic formatting for output.
//!
//! Supports pretty (human-readable), JSON, and compact output formats.

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
fn format_pretty(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = String::new();

    for diag in diagnostics {
        let (line, col) = offset_to_line_col(source_text, diag.span.start);
        let severity_str = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Suggestion => "suggestion",
        };

        output.push_str(&format!(
            "  {severity_str}[{rule}]: {message}\n",
            rule = diag.rule_name,
            message = diag.message,
        ));
        output.push_str(&format!(
            "    --> {path}:{line}:{col}\n",
            path = file_path.display(),
        ));

        if let Some(help) = &diag.help {
            output.push_str(&format!("    help: {help}\n"));
        }

        output.push('\n');
    }

    output
}

/// Format diagnostics as JSON.
fn format_json(diagnostics: &[Diagnostic], file_path: &Path) -> String {
    let mut entries = Vec::new();
    for diag in diagnostics {
        // Build a JSON object with file_path included.
        let entry = serde_json::json!({
            "file": file_path.display().to_string(),
            "rule": diag.rule_name,
            "message": diag.message,
            "severity": diag.severity,
            "span": { "start": diag.span.start, "end": diag.span.end },
            "help": diag.help,
        });
        entries.push(entry);
    }
    // Return one JSON object per line for streaming compatibility.
    entries
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format diagnostics in compact single-line form.
fn format_compact(diagnostics: &[Diagnostic], file_path: &Path) -> String {
    let mut output = String::new();
    for diag in diagnostics {
        let severity_char = match diag.severity {
            Severity::Error => 'E',
            Severity::Warning => 'W',
            Severity::Suggestion => 'S',
        };
        output.push_str(&format!(
            "{path}:{start}-{end} {sev} [{rule}] {message}\n",
            path = file_path.display(),
            start = diag.span.start,
            end = diag.span.end,
            sev = severity_char,
            rule = diag.rule_name,
            message = diag.message,
        ));
    }
    output
}

/// Convert a byte offset to 1-based line and column numbers.
fn offset_to_line_col(source: &str, offset: u32) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    #[allow(clippy::indexing_slicing)]
    for (i, ch) in source.char_indices() {
        if i >= offset.try_into().unwrap_or(usize::MAX) {
            break;
        }
        if ch == '\n' {
            line = line.wrapping_add(1);
            col = 1;
        } else {
            col = col.wrapping_add(1);
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
        assert_eq!(offset_to_line_col(source, 4), (2, 1), "start of second line");
        assert_eq!(offset_to_line_col(source, 5), (2, 2), "second char of second line");
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
        assert!(output.contains("warning[no-debugger]"), "should contain severity and rule");
        assert!(output.contains("test.js:1:1"), "should contain file location");
    }

    #[test]
    fn test_format_json() {
        let diag = make_diag("no-debugger", "bad", Severity::Error);
        let output = format_json(&[diag], Path::new("test.js"));
        assert!(output.contains("\"rule\":\"no-debugger\""), "json should contain rule name");
    }
}
