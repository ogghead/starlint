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
    /// Count-only mode: no diagnostic output, just summary counts.
    Count,
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
        OutputFormat::Count => String::new(),
    }
}

/// Write diagnostics for a single file directly to a writer.
///
/// Avoids building an intermediate `String` — formats directly into the writer.
/// For [`OutputFormat::Count`], this is a no-op.
pub fn write_diagnostics(
    writer: &mut impl std::io::Write,
    diagnostics: &[Diagnostic],
    source_text: &str,
    file_path: &Path,
    format: OutputFormat,
) -> std::io::Result<()> {
    match format {
        OutputFormat::Pretty => write_pretty(writer, diagnostics, source_text, file_path),
        OutputFormat::Json => write_json(writer, diagnostics, file_path),
        OutputFormat::Compact => write_compact(writer, diagnostics, file_path),
        OutputFormat::Count => Ok(()),
    }
}

/// Format diagnostics in human-readable form.
#[allow(clippy::let_underscore_must_use)] // writeln! to String is infallible
fn format_pretty(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = String::new();
    let index = LineIndex::new(source_text);

    for diag in diagnostics {
        let (line, col) = index.offset_to_line_col(source_text, diag.span.start);
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

/// Write diagnostics in human-readable form directly to a writer.
fn write_pretty(
    writer: &mut impl std::io::Write,
    diagnostics: &[Diagnostic],
    source_text: &str,
    file_path: &Path,
) -> std::io::Result<()> {
    let index = LineIndex::new(source_text);

    for diag in diagnostics {
        let (line, col) = index.offset_to_line_col(source_text, diag.span.start);
        let severity_str = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Suggestion => "suggestion",
        };

        writeln!(
            writer,
            "  {severity_str}[{rule}]: {message}",
            rule = diag.rule_name,
            message = diag.message,
        )?;
        writeln!(
            writer,
            "    --> {path}:{line}:{col}",
            path = file_path.display(),
        )?;

        if let Some(help) = &diag.help {
            writeln!(writer, "    help: {help}")?;
        }

        writeln!(writer)?;
    }

    Ok(())
}

/// Format diagnostics as newline-delimited JSON (NDJSON).
///
/// Each diagnostic is emitted as a standalone JSON object on its own line,
/// rather than wrapped in a JSON array. This is compatible with tools like
/// `jq` and line-oriented log processors.
fn format_json(diagnostics: &[Diagnostic], file_path: &Path) -> String {
    let mut output = Vec::new();
    write_json(&mut output, diagnostics, file_path).ok();
    String::from_utf8(output).unwrap_or_default()
}

/// Write diagnostics as newline-delimited JSON directly to a writer.
fn write_json(
    writer: &mut impl std::io::Write,
    diagnostics: &[Diagnostic],
    file_path: &Path,
) -> std::io::Result<()> {
    let file_str = file_path.display().to_string();
    for (i, diag) in diagnostics.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }
        let entry = serde_json::json!({
            "file": file_str,
            "rule": diag.rule_name,
            "message": diag.message,
            "severity": diag.severity,
            "span": { "start": diag.span.start, "end": diag.span.end },
            "help": diag.help,
        });
        match serde_json::to_writer(&mut *writer, &entry) {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!(
                    "failed to serialize diagnostic for rule '{}': {err}",
                    diag.rule_name
                );
            }
        }
    }
    Ok(())
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

/// Write diagnostics in compact single-line form directly to a writer.
fn write_compact(
    writer: &mut impl std::io::Write,
    diagnostics: &[Diagnostic],
    file_path: &Path,
) -> std::io::Result<()> {
    for diag in diagnostics {
        let severity_char = match diag.severity {
            Severity::Error => 'E',
            Severity::Warning => 'W',
            Severity::Suggestion => 'S',
        };
        writeln!(
            writer,
            "{path}:{start}-{end} {sev} [{rule}] {message}",
            path = file_path.display(),
            start = diag.span.start,
            end = diag.span.end,
            sev = severity_char,
            rule = diag.rule_name,
            message = diag.message,
        )?;
    }
    Ok(())
}

/// Pre-computed index of newline byte offsets for O(log N) line/column lookups.
///
/// Built once per file, then shared across all diagnostics for that file.
struct LineIndex {
    /// Byte offsets of the start of each line. `line_starts[0]` is always 0.
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Build a line index from source text.
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (i, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                let offset = u32::try_from(i).unwrap_or(u32::MAX);
                line_starts.push(offset.saturating_add(1));
            }
        }
        Self { line_starts }
    }

    /// Convert a byte offset to 1-based (line, column).
    ///
    /// Column is measured in UTF-8 characters (not bytes) from the start of the line,
    /// matching the previous `offset_to_line_col` behavior.
    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit platforms
    fn offset_to_line_col(&self, source: &str, offset: u32) -> (usize, usize) {
        // Binary search for the line containing this offset.
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(insert) => insert.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line_idx).copied().unwrap_or(0);

        // Count characters (not bytes) from line start to offset for the column.
        let start = line_start as usize;
        let end = (offset as usize).min(source.len());
        let col = if start <= end {
            source
                .get(start..end)
                .map_or(1, |slice| slice.chars().count().saturating_add(1))
        } else {
            1
        };

        (line_idx.saturating_add(1), col)
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use starlint_plugin_sdk::diagnostic::Span;

    /// Test helper: build a `LineIndex` and look up a single offset.
    fn offset_to_line_col(source: &str, offset: u32) -> (usize, usize) {
        LineIndex::new(source).offset_to_line_col(source, offset)
    }

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

    #[test]
    fn test_format_diagnostics_dispatches() {
        let diag = make_diag("test/rule", "msg", Severity::Error);
        let diags = &[diag];
        let source = "let x = 1;";
        let path = Path::new("test.js");

        let pretty = format_diagnostics(diags, source, path, OutputFormat::Pretty);
        assert!(
            pretty.contains("error[test/rule]"),
            "pretty format should contain error prefix"
        );

        let json = format_diagnostics(diags, source, path, OutputFormat::Json);
        assert!(
            json.contains("\"rule\":\"test/rule\""),
            "json format should contain rule"
        );

        let compact = format_diagnostics(diags, source, path, OutputFormat::Compact);
        assert!(
            compact.contains("E [test/rule]"),
            "compact format should contain severity char"
        );

        let count = format_diagnostics(diags, source, path, OutputFormat::Count);
        assert!(count.is_empty(), "count format should be empty");
    }

    #[test]
    fn test_write_diagnostics_pretty() {
        let diag = make_diag("test/rule", "bad code", Severity::Warning);
        let mut buf = Vec::new();
        write_diagnostics(
            &mut buf,
            &[diag],
            "let x = 1;",
            Path::new("test.js"),
            OutputFormat::Pretty,
        )
        .ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(
            output.contains("warning[test/rule]"),
            "should contain warning"
        );
        assert!(output.contains("test.js:1:1"), "should contain location");
    }

    #[test]
    fn test_write_diagnostics_json() {
        let diag = make_diag("test/rule", "msg", Severity::Error);
        let mut buf = Vec::new();
        write_diagnostics(
            &mut buf,
            &[diag],
            "x;",
            Path::new("test.js"),
            OutputFormat::Json,
        )
        .ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(
            output.contains("\"rule\":\"test/rule\""),
            "json should contain rule"
        );
    }

    #[test]
    fn test_write_diagnostics_compact() {
        let diag = make_diag("test/rule", "msg", Severity::Error);
        let mut buf = Vec::new();
        write_diagnostics(
            &mut buf,
            &[diag],
            "x;",
            Path::new("test.js"),
            OutputFormat::Compact,
        )
        .ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(
            output.contains("E [test/rule]"),
            "compact should contain severity char"
        );
    }

    #[test]
    fn test_write_diagnostics_count_is_noop() {
        let diag = make_diag("test/rule", "msg", Severity::Error);
        let mut buf = Vec::new();
        let result = write_diagnostics(
            &mut buf,
            &[diag],
            "x;",
            Path::new("test.js"),
            OutputFormat::Count,
        );
        assert!(result.is_ok(), "count format should succeed");
        assert!(buf.is_empty(), "count format should write nothing");
    }

    #[test]
    fn test_write_pretty_with_help() {
        let mut diag = make_diag("test/rule", "msg", Severity::Error);
        diag.help = Some("try this instead".to_owned());
        let mut buf = Vec::new();
        write_pretty(&mut buf, &[diag], "x;", Path::new("test.js")).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(
            output.contains("help: try this instead"),
            "should contain help text"
        );
    }

    #[test]
    fn test_format_pretty_suggestion_severity() {
        let diag = make_diag("test/rule", "msg", Severity::Suggestion);
        let output = format_pretty(&[diag], "x;", Path::new("test.js"));
        assert!(
            output.contains("suggestion[test/rule]"),
            "should format suggestion severity"
        );
    }

    #[test]
    fn test_compact_all_severities() {
        let diags = vec![
            make_diag("r1", "err", Severity::Error),
            make_diag("r2", "warn", Severity::Warning),
            make_diag("r3", "sugg", Severity::Suggestion),
        ];
        let output = format_compact(&diags, Path::new("test.js"));
        assert!(output.contains(" E [r1]"), "should have E for error");
        assert!(output.contains(" W [r2]"), "should have W for warning");
        assert!(output.contains(" S [r3]"), "should have S for suggestion");
    }

    #[test]
    fn test_write_compact_all_severities() {
        let diags = vec![
            make_diag("r1", "err", Severity::Error),
            make_diag("r2", "warn", Severity::Warning),
            make_diag("r3", "sugg", Severity::Suggestion),
        ];
        let mut buf = Vec::new();
        write_compact(&mut buf, &diags, Path::new("test.js")).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains(" E [r1]"), "should have E");
        assert!(output.contains(" W [r2]"), "should have W");
        assert!(output.contains(" S [r3]"), "should have S");
    }

    #[test]
    fn test_format_json_multiple_diagnostics() {
        let diags = vec![
            make_diag("r1", "first", Severity::Error),
            make_diag("r2", "second", Severity::Warning),
        ];
        let output = format_json(&diags, Path::new("test.js"));
        assert!(output.contains("\"r1\""), "should contain first rule");
        assert!(output.contains("\"r2\""), "should contain second rule");
    }

    #[test]
    fn test_write_json_multiple_diagnostics() {
        let diags = vec![
            make_diag("r1", "first", Severity::Error),
            make_diag("r2", "second", Severity::Warning),
        ];
        let mut buf = Vec::new();
        write_json(&mut buf, &diags, Path::new("test.js")).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("\"r1\""), "should contain first rule");
        assert!(output.contains("\"r2\""), "should contain second rule");
    }

    #[test]
    fn test_format_pretty_with_help() {
        let mut diag = make_diag("test/rule", "msg", Severity::Error);
        diag.help = Some("fix it".to_owned());
        let output = format_pretty(&[diag], "x;", Path::new("test.js"));
        assert!(output.contains("help: fix it"), "should contain help text");
    }

    #[test]
    fn test_write_pretty_suggestion_severity() {
        let diag = make_diag("test/rule", "msg", Severity::Suggestion);
        let mut buf = Vec::new();
        write_pretty(&mut buf, &[diag], "x;", Path::new("test.js")).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(
            output.contains("suggestion[test/rule]"),
            "should format suggestion severity"
        );
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(
            OutputFormat::default(),
            OutputFormat::Pretty,
            "default should be Pretty"
        );
    }

    #[test]
    fn test_write_json_with_help() {
        let mut diag = make_diag("test/rule", "msg", Severity::Error);
        diag.help = Some("helpful".to_owned());
        let mut buf = Vec::new();
        write_json(&mut buf, &[diag], Path::new("test.js")).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(
            output.contains("\"help\":\"helpful\""),
            "json should contain help"
        );
    }
}
