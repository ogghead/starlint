//! Conversion between starlint diagnostics and LSP types.
//!
//! Handles byte-offset-to-UTF-16-position conversion and maps
//! starlint `Diagnostic`, `Severity`, and `Fix` types to their LSP equivalents.

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use tower_lsp::lsp_types;

/// Convert a byte offset in `source` to an LSP `Position` (0-based line, UTF-16 column).
///
/// LSP positions use UTF-16 code unit offsets for the character field.
/// This matters for multi-byte characters like emoji (surrogate pairs in UTF-16).
#[must_use]
pub fn byte_offset_to_position(offset: u32, source: &str) -> lsp_types::Position {
    let offset_usize: usize = offset.try_into().unwrap_or(usize::MAX);
    let clamped = offset_usize.min(source.len());

    let mut line: u32 = 0;
    let mut utf16_col: u32 = 0;

    for (i, ch) in source.char_indices() {
        if i >= clamped {
            break;
        }
        if ch == '\n' {
            line = line.saturating_add(1);
            utf16_col = 0;
        } else {
            utf16_col = utf16_col.saturating_add(ch.len_utf16().try_into().unwrap_or(1));
        }
    }

    lsp_types::Position::new(line, utf16_col)
}

/// Convert a starlint `Span` (byte offsets) to an LSP `Range`.
#[must_use]
pub fn span_to_range(span: Span, source: &str) -> lsp_types::Range {
    lsp_types::Range::new(
        byte_offset_to_position(span.start, source),
        byte_offset_to_position(span.end, source),
    )
}

/// Convert a starlint `Severity` to an LSP `DiagnosticSeverity`.
#[must_use]
pub const fn to_lsp_severity(severity: Severity) -> lsp_types::DiagnosticSeverity {
    match severity {
        Severity::Error => lsp_types::DiagnosticSeverity::ERROR,
        Severity::Warning => lsp_types::DiagnosticSeverity::WARNING,
        Severity::Suggestion => lsp_types::DiagnosticSeverity::HINT,
    }
}

/// Convert a starlint `Diagnostic` to an LSP `Diagnostic`.
///
/// Help text is mapped into `related_information` since LSP diagnostics
/// don't have a dedicated help field.
#[must_use]
pub fn to_lsp_diagnostic(diag: &Diagnostic, source: &str) -> lsp_types::Diagnostic {
    let range = span_to_range(diag.span, source);

    lsp_types::Diagnostic {
        range,
        severity: Some(to_lsp_severity(diag.severity)),
        code: Some(lsp_types::NumberOrString::String(diag.rule_name.clone())),
        source: Some("starlint".to_owned()),
        message: if let Some(help) = &diag.help {
            format!("{}\n\nhelp: {help}", diag.message)
        } else {
            diag.message.clone()
        },
        ..Default::default()
    }
}

/// Convert a starlint `Fix` to an LSP `CodeAction`.
///
/// Returns `None` if the diagnostic has no fix.
#[must_use]
pub fn fix_to_code_action(
    diag: &Diagnostic,
    uri: &lsp_types::Url,
    source: &str,
) -> Option<lsp_types::CodeAction> {
    let fix = diag.fix.as_ref()?;

    let mut text_edits = Vec::new();
    for edit in &fix.edits {
        text_edits.push(lsp_types::TextEdit {
            range: span_to_range(edit.span, source),
            new_text: edit.replacement.clone(),
        });
    }

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), text_edits);

    Some(lsp_types::CodeAction {
        title: fix.message.clone(),
        kind: Some(lsp_types::CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![to_lsp_diagnostic(diag, source)]),
        edit: Some(lsp_types::WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        ..Default::default()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests use unwrap for brevity on infallible operations
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use starlint_plugin_sdk::diagnostic::{Edit, Fix};

    /// Parse a URL for testing. All test URLs are known-valid constants.
    fn test_url(s: &str) -> lsp_types::Url {
        lsp_types::Url::parse(s).unwrap()
    }

    #[test]
    fn test_byte_offset_ascii() {
        let source = "abc\ndef\nghi";
        assert_eq!(
            byte_offset_to_position(0, source),
            lsp_types::Position::new(0, 0),
            "start of file"
        );
        assert_eq!(
            byte_offset_to_position(4, source),
            lsp_types::Position::new(1, 0),
            "start of second line"
        );
        assert_eq!(
            byte_offset_to_position(5, source),
            lsp_types::Position::new(1, 1),
            "second char of second line"
        );
    }

    #[test]
    fn test_byte_offset_multibyte() {
        // 'ä' is 2 bytes in UTF-8 but 1 UTF-16 code unit.
        let source = "ä\nb";
        assert_eq!(
            byte_offset_to_position(0, source),
            lsp_types::Position::new(0, 0),
            "start"
        );
        assert_eq!(
            byte_offset_to_position(2, source),
            lsp_types::Position::new(0, 1),
            "after ä (1 UTF-16 unit)"
        );
        assert_eq!(
            byte_offset_to_position(3, source),
            lsp_types::Position::new(1, 0),
            "start of second line"
        );
    }

    #[test]
    fn test_byte_offset_emoji() {
        // '😀' is 4 bytes in UTF-8 and 2 UTF-16 code units (surrogate pair).
        let source = "a😀b";
        assert_eq!(
            byte_offset_to_position(0, source),
            lsp_types::Position::new(0, 0),
            "'a'"
        );
        assert_eq!(
            byte_offset_to_position(1, source),
            lsp_types::Position::new(0, 1),
            "start of emoji"
        );
        assert_eq!(
            byte_offset_to_position(5, source),
            lsp_types::Position::new(0, 3),
            "'b' after emoji (1 + 2 UTF-16 units)"
        );
    }

    #[test]
    fn test_byte_offset_cjk() {
        // '中' is 3 bytes in UTF-8, 1 UTF-16 code unit.
        let source = "中文";
        assert_eq!(
            byte_offset_to_position(0, source),
            lsp_types::Position::new(0, 0),
            "first CJK char"
        );
        assert_eq!(
            byte_offset_to_position(3, source),
            lsp_types::Position::new(0, 1),
            "second CJK char"
        );
    }

    #[test]
    fn test_byte_offset_past_end() {
        let source = "ab";
        let pos = byte_offset_to_position(100, source);
        assert_eq!(
            pos,
            lsp_types::Position::new(0, 2),
            "offset past end clamps to end"
        );
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            to_lsp_severity(Severity::Error),
            lsp_types::DiagnosticSeverity::ERROR,
            "error maps to ERROR"
        );
        assert_eq!(
            to_lsp_severity(Severity::Warning),
            lsp_types::DiagnosticSeverity::WARNING,
            "warning maps to WARNING"
        );
        assert_eq!(
            to_lsp_severity(Severity::Suggestion),
            lsp_types::DiagnosticSeverity::HINT,
            "suggestion maps to HINT"
        );
    }

    #[test]
    fn test_to_lsp_diagnostic_maps_fields() {
        let diag = Diagnostic {
            rule_name: "no-debugger".to_owned(),
            message: "Unexpected debugger statement".to_owned(),
            span: Span::new(0, 9),
            severity: Severity::Error,
            help: Some("Remove the debugger statement".to_owned()),
            fix: None,
            labels: vec![],
        };
        let lsp_diag = to_lsp_diagnostic(&diag, "debugger;");

        assert_eq!(
            lsp_diag.range.start,
            lsp_types::Position::new(0, 0),
            "range start"
        );
        assert_eq!(
            lsp_diag.range.end,
            lsp_types::Position::new(0, 9),
            "range end"
        );
        assert_eq!(
            lsp_diag.severity,
            Some(lsp_types::DiagnosticSeverity::ERROR),
            "severity"
        );
        assert_eq!(
            lsp_diag.code,
            Some(lsp_types::NumberOrString::String("no-debugger".to_owned())),
            "code"
        );
        assert_eq!(lsp_diag.source, Some("starlint".to_owned()), "source");
        assert!(
            lsp_diag
                .message
                .contains("help: Remove the debugger statement"),
            "help text should be in message"
        );
    }

    #[test]
    fn test_fix_to_code_action() {
        let diag = Diagnostic {
            rule_name: "no-extra-semi".to_owned(),
            message: "Unnecessary semicolon".to_owned(),
            span: Span::new(0, 1),
            severity: Severity::Warning,
            help: None,
            fix: Some(Fix {
                message: "Remove semicolon".to_owned(),
                edits: vec![Edit {
                    span: Span::new(0, 1),
                    replacement: String::new(),
                }],
            }),
            labels: vec![],
        };
        let uri = test_url("file:///test.js");
        let maybe_action = fix_to_code_action(&diag, &uri, ";");
        assert!(maybe_action.is_some(), "should produce a code action");

        let action = maybe_action.unwrap();
        assert_eq!(action.title, "Remove semicolon", "action title");
        assert_eq!(
            action.kind,
            Some(lsp_types::CodeActionKind::QUICKFIX),
            "action kind"
        );
        assert!(action.edit.is_some(), "should have workspace edit");
    }

    #[test]
    fn test_fix_to_code_action_none_without_fix() {
        let diag = Diagnostic {
            rule_name: "no-debugger".to_owned(),
            message: "bad".to_owned(),
            span: Span::new(0, 1),
            severity: Severity::Error,
            help: None,
            fix: None,
            labels: vec![],
        };
        let uri = test_url("file:///test.js");
        assert!(
            fix_to_code_action(&diag, &uri, "x").is_none(),
            "no fix means no code action"
        );
    }
}
