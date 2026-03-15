//! Diagnostic formatting for output.
//!
//! Supports pretty (human-readable), JSON, compact, GitHub Actions, GitLab Code
//! Quality, `JUnit` XML, SARIF v2.1.0, and stylish (ESLint-style grouped) output
//! formats.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
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
    /// GitHub Actions workflow command format (`::error`, `::warning`, `::notice`).
    Github,
    /// GitLab Code Quality JSON array format.
    Gitlab,
    /// `JUnit` XML format.
    Junit,
    /// SARIF v2.1.0 JSON format.
    Sarif,
    /// ESLint-style grouped format (grouped by file with summary).
    Stylish,
}

/// Returns `true` for formats that produce a single document from all files
/// (as opposed to per-file streaming).
#[must_use]
pub const fn is_document_format(format: OutputFormat) -> bool {
    matches!(
        format,
        OutputFormat::Gitlab | OutputFormat::Sarif | OutputFormat::Junit
    )
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
        OutputFormat::Github => format_github(diagnostics, source_text, file_path),
        OutputFormat::Gitlab | OutputFormat::Sarif | OutputFormat::Junit => {
            // Document formats are handled via write_document_diagnostics.
            String::new()
        }
        OutputFormat::Stylish => format_stylish(diagnostics, source_text, file_path),
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
        OutputFormat::Github => write_github(writer, diagnostics, source_text, file_path),
        OutputFormat::Gitlab | OutputFormat::Sarif | OutputFormat::Junit => {
            // Document formats are handled via write_document_diagnostics.
            Ok(())
        }
        OutputFormat::Stylish => write_stylish(writer, diagnostics, source_text, file_path),
    }
}

/// Format diagnostics in human-readable form.
fn format_pretty(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = Vec::new();
    write_pretty(&mut output, diagnostics, source_text, file_path).ok();
    String::from_utf8(output).unwrap_or_default()
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

/// Lightweight struct for direct JSON serialization without intermediate `Value`.
#[derive(Serialize)]
struct JsonDiagnostic<'a> {
    /// File path.
    file: &'a str,
    /// Rule name.
    rule: &'a str,
    /// Diagnostic message.
    message: &'a str,
    /// Severity level.
    severity: &'a Severity,
    /// Source span.
    span: JsonSpan,
    /// Optional help text.
    help: Option<&'a str>,
}

/// Span serialization helper.
#[derive(Serialize)]
struct JsonSpan {
    /// Start byte offset.
    start: u32,
    /// End byte offset.
    end: u32,
}

/// Write diagnostics as newline-delimited JSON directly to a writer.
///
/// Serializes directly from a typed struct instead of building an intermediate
/// `serde_json::Value`, avoiding per-diagnostic allocation overhead.
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
        let entry = JsonDiagnostic {
            file: &file_str,
            rule: &diag.rule_name,
            message: &diag.message,
            severity: &diag.severity,
            span: JsonSpan {
                start: diag.span.start,
                end: diag.span.end,
            },
            help: diag.help.as_deref(),
        };
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
fn format_compact(diagnostics: &[Diagnostic], file_path: &Path) -> String {
    let mut output = Vec::new();
    write_compact(&mut output, diagnostics, file_path).ok();
    String::from_utf8(output).unwrap_or_default()
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

// ── GitHub Actions format ─────────────────────────────────────────────

/// Format diagnostics as GitHub Actions workflow commands.
fn format_github(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = Vec::new();
    write_github(&mut output, diagnostics, source_text, file_path).ok();
    String::from_utf8(output).unwrap_or_default()
}

/// Write diagnostics as GitHub Actions workflow commands directly to a writer.
///
/// Format: `::error file={path},line={line},col={col}::{message} [{rule}]`
/// Severity mapping: Error -> `error`, Warning -> `warning`, Suggestion -> `notice`.
fn write_github(
    writer: &mut impl std::io::Write,
    diagnostics: &[Diagnostic],
    source_text: &str,
    file_path: &Path,
) -> std::io::Result<()> {
    let index = LineIndex::new(source_text);

    for diag in diagnostics {
        let (line, col) = index.offset_to_line_col(source_text, diag.span.start);
        let level = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Suggestion => "notice",
        };

        writeln!(
            writer,
            "::{level} file={path},line={line},col={col}::{message} [{rule}]",
            path = file_path.display(),
            message = diag.message,
            rule = diag.rule_name,
        )?;
    }

    Ok(())
}

// ── GitLab Code Quality format ────────────────────────────────────────

/// A single entry in the GitLab Code Quality JSON array.
#[derive(Serialize)]
struct GitlabEntry<'a> {
    /// Diagnostic message.
    description: &'a str,
    /// Rule name.
    check_name: &'a str,
    /// Unique fingerprint for deduplication.
    fingerprint: String,
    /// Severity level.
    severity: &'a str,
    /// Source location.
    location: GitlabLocation<'a>,
}

/// Location information for a GitLab Code Quality entry.
#[derive(Serialize)]
struct GitlabLocation<'a> {
    /// File path.
    path: &'a str,
    /// Line range.
    lines: GitlabLines,
}

/// Line range for a GitLab Code Quality entry.
#[derive(Serialize)]
struct GitlabLines {
    /// Beginning line number.
    begin: usize,
}

/// Write diagnostics from multiple files as a GitLab Code Quality JSON array.
///
/// This is a document-level format: all diagnostics across all files are
/// collected into a single JSON array.
pub fn write_document_gitlab(
    writer: &mut impl std::io::Write,
    all_results: &[(Vec<Diagnostic>, String, std::path::PathBuf)],
) -> std::io::Result<()> {
    let mut entries: Vec<GitlabEntry<'_>> = Vec::new();

    for (diagnostics, source_text, file_path) in all_results {
        let index = LineIndex::new(source_text);
        let file_str = file_path.display().to_string();

        for diag in diagnostics {
            let (line, _col) = index.offset_to_line_col(source_text, diag.span.start);
            let severity = match diag.severity {
                Severity::Error => "critical",
                Severity::Warning => "major",
                Severity::Suggestion => "minor",
            };
            let fingerprint = format!("{}:{}:{}", file_str, diag.rule_name, line);

            entries.push(GitlabEntry {
                description: &diag.message,
                check_name: &diag.rule_name,
                fingerprint,
                severity,
                location: GitlabLocation {
                    path: file_path.to_str().unwrap_or(""),
                    lines: GitlabLines { begin: line },
                },
            });
        }
    }

    match serde_json::to_writer_pretty(writer, &entries) {
        Ok(()) => {}
        Err(err) => {
            tracing::warn!("failed to serialize GitLab output: {err}");
        }
    }
    Ok(())
}

/// Format diagnostics for a single file as GitLab Code Quality JSON.
///
/// Primarily used for testing; production use goes through
/// [`write_document_gitlab`].
#[cfg(test)]
fn format_gitlab(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = Vec::new();
    let data = vec![(
        diagnostics.to_vec(),
        source_text.to_owned(),
        file_path.to_path_buf(),
    )];
    write_document_gitlab(&mut output, &data).ok();
    String::from_utf8(output).unwrap_or_default()
}

// ── `JUnit` XML format ──────────────────────────────────────────────────

/// Escape a string for safe inclusion in XML text content or attribute values.
#[must_use]
fn xml_escape(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            other => result.push(other),
        }
    }
    result
}

/// Internal representation of a single `JUnit` test case for XML generation.
struct JunitTestCase {
    /// Rule name for the test case.
    rule: String,
    /// File path as classname.
    classname: String,
    /// Severity label.
    severity: String,
    /// Diagnostic message.
    message: String,
    /// Full detail string (`path:line:col message`).
    detail: String,
}

/// Write diagnostics from multiple files as a `JUnit` XML document.
///
/// This is a document-level format: all diagnostics across all files are
/// collected into a single `<testsuites>` element.
pub fn write_document_junit(
    writer: &mut impl std::io::Write,
    all_results: &[(Vec<Diagnostic>, String, std::path::PathBuf)],
) -> std::io::Result<()> {
    let mut total_tests = 0usize;
    let mut total_failures = 0usize;
    let mut cases: Vec<JunitTestCase> = Vec::new();

    for (diagnostics, source_text, file_path) in all_results {
        let index = LineIndex::new(source_text);
        let file_str = file_path.display().to_string();

        for diag in diagnostics {
            let (line, col) = index.offset_to_line_col(source_text, diag.span.start);
            let severity = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Suggestion => "suggestion",
            };

            total_tests = total_tests.saturating_add(1);
            total_failures = total_failures.saturating_add(1);

            cases.push(JunitTestCase {
                rule: diag.rule_name.clone(),
                classname: file_str.clone(),
                severity: severity.to_owned(),
                message: diag.message.clone(),
                detail: format!("{file_str}:{line}:{col} {}", diag.message),
            });
        }
    }

    writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(writer, "<testsuites>")?;
    writeln!(
        writer,
        "  <testsuite name=\"starlint\" tests=\"{total_tests}\" failures=\"{total_failures}\">"
    )?;

    for case in &cases {
        writeln!(
            writer,
            "    <testcase name=\"{rule}\" classname=\"{classname}\">",
            rule = xml_escape(&case.rule),
            classname = xml_escape(&case.classname),
        )?;
        writeln!(
            writer,
            "      <failure message=\"{message}\" type=\"{severity}\">{detail}</failure>",
            message = xml_escape(&case.message),
            severity = xml_escape(&case.severity),
            detail = xml_escape(&case.detail),
        )?;
        writeln!(writer, "    </testcase>")?;
    }

    writeln!(writer, "  </testsuite>")?;
    writeln!(writer, "</testsuites>")?;
    Ok(())
}

/// Format diagnostics for a single file as `JUnit` XML.
///
/// Primarily used for testing; production use goes through
/// [`write_document_junit`].
#[cfg(test)]
fn format_junit(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = Vec::new();
    let data = vec![(
        diagnostics.to_vec(),
        source_text.to_owned(),
        file_path.to_path_buf(),
    )];
    write_document_junit(&mut output, &data).ok();
    String::from_utf8(output).unwrap_or_default()
}

// ── SARIF v2.1.0 format ──────────────────────────────────────────────

/// Write diagnostics from multiple files as a SARIF v2.1.0 JSON document.
///
/// This is a document-level format: all diagnostics across all files are
/// collected into a single SARIF `runs` array.
pub fn write_document_sarif(
    writer: &mut impl std::io::Write,
    all_results: &[(Vec<Diagnostic>, String, std::path::PathBuf)],
) -> std::io::Result<()> {
    // Collect unique rules for the driver rules array.
    let mut rule_names: Vec<String> = Vec::new();
    let mut rule_index_map: HashMap<String, usize> = HashMap::new();
    let mut results_array: Vec<serde_json::Value> = Vec::new();

    for (diagnostics, source_text, file_path) in all_results {
        let index = LineIndex::new(source_text);
        let file_str = file_path.display().to_string();

        for diag in diagnostics {
            let (line, col) = index.offset_to_line_col(source_text, diag.span.start);
            let rule_position = if let Some(&idx) = rule_index_map.get(&diag.rule_name) {
                idx
            } else {
                let idx = rule_names.len();
                rule_names.push(diag.rule_name.clone());
                rule_index_map.insert(diag.rule_name.clone(), idx);
                idx
            };

            let level = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Suggestion => "note",
            };

            let result_obj = serde_json::json!({
                "ruleId": diag.rule_name,
                "ruleIndex": rule_position,
                "level": level,
                "message": {
                    "text": diag.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_str
                        },
                        "region": {
                            "startLine": line,
                            "startColumn": col
                        }
                    }
                }]
            });
            results_array.push(result_obj);
        }
    }

    let rules_array: Vec<serde_json::Value> = rule_names
        .iter()
        .map(|id| {
            serde_json::json!({
                "id": id
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "starlint",
                    "rules": rules_array
                }
            },
            "results": results_array
        }]
    });

    match serde_json::to_writer_pretty(writer, &sarif) {
        Ok(()) => {}
        Err(err) => {
            tracing::warn!("failed to serialize SARIF output: {err}");
        }
    }
    Ok(())
}

/// Format diagnostics for a single file as SARIF v2.1.0 JSON.
///
/// Primarily used for testing; production use goes through
/// [`write_document_sarif`].
#[cfg(test)]
fn format_sarif(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = Vec::new();
    let data = vec![(
        diagnostics.to_vec(),
        source_text.to_owned(),
        file_path.to_path_buf(),
    )];
    write_document_sarif(&mut output, &data).ok();
    String::from_utf8(output).unwrap_or_default()
}

// ── Stylish (ESLint-style) format ─────────────────────────────────────

/// Format diagnostics in ESLint-style grouped format.
fn format_stylish(diagnostics: &[Diagnostic], source_text: &str, file_path: &Path) -> String {
    let mut output = Vec::new();
    write_stylish(&mut output, diagnostics, source_text, file_path).ok();
    String::from_utf8(output).unwrap_or_default()
}

/// Write diagnostics in ESLint-style grouped format directly to a writer.
///
/// Groups diagnostics under the file path and appends a summary line with
/// total problem, error, and warning counts.
fn write_stylish(
    writer: &mut impl std::io::Write,
    diagnostics: &[Diagnostic],
    source_text: &str,
    file_path: &Path,
) -> std::io::Result<()> {
    if diagnostics.is_empty() {
        return Ok(());
    }

    let index = LineIndex::new(source_text);

    writeln!(writer, "{}", file_path.display())?;

    let mut error_count = 0usize;
    let mut warning_count = 0usize;

    for diag in diagnostics {
        let (line, col) = index.offset_to_line_col(source_text, diag.span.start);
        let severity_str = match diag.severity {
            Severity::Error => {
                error_count = error_count.saturating_add(1);
                "error"
            }
            Severity::Warning => {
                warning_count = warning_count.saturating_add(1);
                "warning"
            }
            Severity::Suggestion => "suggestion",
        };

        writeln!(
            writer,
            "  {line}:{col}  {severity_str}  {message}  {rule}",
            message = diag.message,
            rule = diag.rule_name,
        )?;
    }

    writeln!(writer)?;

    let total = error_count.saturating_add(warning_count);
    writeln!(
        writer,
        "X {total} problem(s) ({error_count} error(s), {warning_count} warning(s))"
    )?;

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
    ///
    /// Pre-counts newlines to allocate the right capacity up front,
    /// avoiding repeated reallocation.
    fn new(source: &str) -> Self {
        let newline_count = bytecount(source.as_bytes(), b'\n');
        let mut line_starts = Vec::with_capacity(newline_count.saturating_add(1));
        line_starts.push(0u32);
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
    /// Column is measured in UTF-8 characters (not bytes) from the start of the line.
    /// Uses a fast path when the slice is all ASCII (byte count == char count).
    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit platforms
    fn offset_to_line_col(&self, source: &str, offset: u32) -> (usize, usize) {
        // Binary search for the line containing this offset.
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(insert) => insert.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line_idx).copied().unwrap_or(0);

        let start = line_start as usize;
        let end = (offset as usize).min(source.len());
        let col = if start <= end {
            source.get(start..end).map_or(1, |slice| {
                // Fast path: if byte length == char count, the slice is ASCII-only.
                // Most JS/TS source is ASCII, so this avoids the expensive char iteration.
                if slice.is_ascii() {
                    slice.len().saturating_add(1)
                } else {
                    slice.chars().count().saturating_add(1)
                }
            })
        } else {
            1
        };

        (line_idx.saturating_add(1), col)
    }
}

/// Count occurrences of a byte in a slice.
#[allow(clippy::naive_bytecount)] // Avoiding extra dependency; compiler auto-vectorizes this
fn bytecount(haystack: &[u8], needle: u8) -> usize {
    haystack.iter().filter(|&&b| b == needle).count()
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

        let github = format_diagnostics(diags, source, path, OutputFormat::Github);
        assert!(
            github.contains("::error"),
            "github format should contain ::error"
        );

        // Document formats return empty from format_diagnostics (handled via
        // write_document_* functions in the CLI layer).
        let gitlab = format_diagnostics(diags, source, path, OutputFormat::Gitlab);
        assert!(
            gitlab.is_empty(),
            "gitlab via format_diagnostics should be empty (document format)"
        );

        let junit = format_diagnostics(diags, source, path, OutputFormat::Junit);
        assert!(
            junit.is_empty(),
            "junit via format_diagnostics should be empty (document format)"
        );

        let sarif = format_diagnostics(diags, source, path, OutputFormat::Sarif);
        assert!(
            sarif.is_empty(),
            "sarif via format_diagnostics should be empty (document format)"
        );

        let stylish = format_diagnostics(diags, source, path, OutputFormat::Stylish);
        assert!(
            stylish.contains("test.js"),
            "stylish format should contain file path"
        );
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

    // ── GitHub Actions format tests ───────────────────────────────────

    #[test]
    fn test_format_github() {
        let diags = vec![
            make_diag("no-debugger", "Unexpected debugger", Severity::Error),
            make_diag("no-console", "Unexpected console", Severity::Warning),
            make_diag("prefer-const", "Use const", Severity::Suggestion),
        ];
        let output = format_github(
            &diags,
            "debugger;\nconsole.log();\nlet x = 1;",
            Path::new("test.js"),
        );
        assert!(
            output.contains("::error file=test.js,line=1,col=1::Unexpected debugger [no-debugger]"),
            "should contain error annotation: {output}"
        );
        assert!(
            output.contains("::warning file=test.js,line=1,col=1::Unexpected console [no-console]"),
            "should contain warning annotation: {output}"
        );
        assert!(
            output.contains("::notice file=test.js,line=1,col=1::Use const [prefer-const]"),
            "should contain notice annotation for suggestion: {output}"
        );
    }

    // ── GitLab Code Quality format tests ──────────────────────────────

    #[test]
    fn test_format_gitlab() {
        let diags = vec![
            make_diag("no-debugger", "Unexpected debugger", Severity::Error),
            make_diag("no-console", "Unexpected console", Severity::Warning),
        ];
        let output = format_gitlab(&diags, "debugger;", Path::new("test.js"));
        assert!(
            output.contains("\"check_name\": \"no-debugger\""),
            "should contain check_name: {output}"
        );
        assert!(
            output.contains("\"severity\": \"critical\""),
            "error should map to critical: {output}"
        );
        assert!(
            output.contains("\"severity\": \"major\""),
            "warning should map to major: {output}"
        );
        assert!(
            output.contains("\"description\": \"Unexpected debugger\""),
            "should contain description: {output}"
        );
        assert!(
            output.contains("\"fingerprint\""),
            "should contain fingerprint: {output}"
        );

        // Verify it is a valid JSON array.
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
        assert!(parsed.is_ok(), "should be valid JSON: {output}");
        assert!(
            parsed.ok().is_some_and(|v| v.is_array()),
            "should be a JSON array"
        );
    }

    // ── `JUnit` XML format tests ────────────────────────────────────────

    #[test]
    fn test_format_junit() {
        let diags = vec![
            make_diag("no-debugger", "Unexpected debugger", Severity::Error),
            make_diag("no-console", "Unexpected console", Severity::Warning),
        ];
        let output = format_junit(&diags, "debugger;", Path::new("test.js"));
        assert!(
            output.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "should contain XML declaration: {output}"
        );
        assert!(
            output.contains("<testsuites>"),
            "should contain testsuites element: {output}"
        );
        assert!(
            output.contains("tests=\"2\""),
            "should have 2 tests: {output}"
        );
        assert!(
            output.contains("failures=\"2\""),
            "should have 2 failures: {output}"
        );
        assert!(
            output.contains("name=\"no-debugger\""),
            "should contain rule name in testcase: {output}"
        );
        assert!(
            output.contains("classname=\"test.js\""),
            "should contain file as classname: {output}"
        );
        assert!(
            output.contains("type=\"error\""),
            "should contain severity type: {output}"
        );
        assert!(
            output.contains("</testsuites>"),
            "should close testsuites: {output}"
        );
    }

    #[test]
    fn test_format_junit_xml_escaping() {
        let diag = make_diag("rule", "Use <div> & \"quotes\"", Severity::Error);
        let output = format_junit(&[diag], "x;", Path::new("test.js"));
        assert!(
            output.contains("&lt;div&gt;"),
            "should escape angle brackets: {output}"
        );
        assert!(
            output.contains("&amp;"),
            "should escape ampersand: {output}"
        );
        assert!(
            output.contains("&quot;quotes&quot;"),
            "should escape quotes: {output}"
        );
    }

    // ── SARIF v2.1.0 format tests ────────────────────────────────────

    #[test]
    fn test_format_sarif() {
        let diags = vec![
            make_diag("no-debugger", "Unexpected debugger", Severity::Error),
            make_diag("no-console", "Unexpected console", Severity::Warning),
        ];
        let output = format_sarif(&diags, "debugger;\nconsole.log();", Path::new("test.js"));
        assert!(
            output.contains("\"version\": \"2.1.0\""),
            "should contain SARIF version: {output}"
        );
        assert!(
            output.contains("sarif-schema-2.1.0.json"),
            "should contain schema reference: {output}"
        );
        assert!(
            output.contains("\"name\": \"starlint\""),
            "should contain tool name: {output}"
        );
        assert!(
            output.contains("\"ruleId\": \"no-debugger\""),
            "should contain ruleId: {output}"
        );
        assert!(
            output.contains("\"level\": \"error\""),
            "should contain error level: {output}"
        );
        assert!(
            output.contains("\"level\": \"warning\""),
            "should contain warning level: {output}"
        );

        // Verify it is valid JSON with the expected structure.
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
        assert!(parsed.is_ok(), "should be valid JSON: {output}");
        assert!(
            parsed.ok().and_then(|v| v.get("runs").cloned()).is_some(),
            "should have runs key"
        );
    }

    // ── Stylish format tests ──────────────────────────────────────────

    #[test]
    fn test_format_stylish() {
        let diags = vec![
            make_diag("no-debugger", "Unexpected debugger", Severity::Error),
            make_diag("no-console", "Unexpected console", Severity::Warning),
        ];
        let output = format_stylish(&diags, "debugger;\nconsole.log();", Path::new("test.js"));
        assert!(
            output.contains("test.js"),
            "should contain file path: {output}"
        );
        assert!(
            output.contains("error"),
            "should contain error severity: {output}"
        );
        assert!(
            output.contains("warning"),
            "should contain warning severity: {output}"
        );
        assert!(
            output.contains("Unexpected debugger"),
            "should contain error message: {output}"
        );
        assert!(
            output.contains("no-debugger"),
            "should contain rule name: {output}"
        );
        assert!(
            output.contains("X 2 problem(s) (1 error(s), 1 warning(s))"),
            "should contain summary line: {output}"
        );
    }

    #[test]
    fn test_format_stylish_empty() {
        let output = format_stylish(&[], "x;", Path::new("test.js"));
        assert!(
            output.is_empty(),
            "empty diagnostics should produce no output"
        );
    }

    // ── is_document_format ────────────────────────────────────────────

    #[test]
    fn test_is_document_format() {
        assert!(!is_document_format(OutputFormat::Pretty));
        assert!(!is_document_format(OutputFormat::Json));
        assert!(!is_document_format(OutputFormat::Compact));
        assert!(!is_document_format(OutputFormat::Count));
        assert!(!is_document_format(OutputFormat::Github));
        assert!(is_document_format(OutputFormat::Gitlab));
        assert!(is_document_format(OutputFormat::Junit));
        assert!(is_document_format(OutputFormat::Sarif));
        assert!(!is_document_format(OutputFormat::Stylish));
    }

    // ── xml_escape ────────────────────────────────────────────────────

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("<div>"), "&lt;div&gt;");
        assert_eq!(xml_escape("a&b"), "a&amp;b");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(xml_escape("it's"), "it&apos;s");
    }
}
