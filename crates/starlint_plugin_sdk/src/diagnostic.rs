//! Diagnostic types produced by lint rules.
//!
//! Both native Rust rules and WASM plugin rules produce [`Diagnostic`] values.
//! The linter engine collects and formats these uniformly.

use serde::{Deserialize, Serialize};

/// A byte-offset span in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: u32,
    /// Exclusive end byte offset.
    pub end: u32,
}

impl Span {
    /// Create a new span from start (inclusive) to end (exclusive).
    #[must_use]
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

/// Severity of a lint diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A warning that does not cause a non-zero exit code by default.
    Warning,
    /// An error that causes a non-zero exit code.
    Error,
    /// A suggestion for improvement (informational).
    Suggestion,
}

/// A lint diagnostic emitted by a rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Fully qualified rule name (e.g. "storybook/default-exports").
    pub rule_name: String,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Primary source span of the issue.
    pub span: Span,
    /// Severity level.
    pub severity: Severity,
    /// Optional help text with a suggestion for fixing.
    pub help: Option<String>,
    /// Optional auto-fix.
    pub fix: Option<Fix>,
    /// Additional labeled spans for context.
    pub labels: Vec<Label>,
}

/// An auto-fix consisting of one or more text edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    /// Human-readable description of the fix.
    pub message: String,
    /// Ordered list of text edits to apply.
    pub edits: Vec<Edit>,
}

/// A single text edit (replace span contents with new text).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    /// Span to replace.
    pub span: Span,
    /// Replacement text.
    pub replacement: String,
}

/// A labeled source span for additional context in diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    /// Source span.
    pub span: Span,
    /// Label message.
    pub message: String,
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10, "start should be 10");
        assert_eq!(span.end, 20, "end should be 20");
    }

    #[test]
    fn test_diagnostic_serialization_roundtrip() {
        let diag = Diagnostic {
            rule_name: "test/rule".to_owned(),
            message: "something is wrong".to_owned(),
            span: Span::new(0, 5),
            severity: Severity::Error,
            help: Some("fix it".to_owned()),
            fix: None,
            labels: vec![],
        };

        let json = serde_json::to_string(&diag).ok();
        assert!(json.is_some(), "serialization should succeed");

        let roundtrip: Result<Diagnostic, _> =
            serde_json::from_str(json.as_deref().unwrap_or(""));
        assert!(roundtrip.is_ok(), "deserialization should succeed");
        assert_eq!(
            roundtrip.ok(),
            Some(diag),
            "roundtrip should produce the same diagnostic"
        );
    }

    #[test]
    fn test_severity_serde() {
        let json = serde_json::to_string(&Severity::Warning).ok();
        assert_eq!(
            json.as_deref(),
            Some("\"warning\""),
            "warning should serialize as lowercase"
        );
    }
}
