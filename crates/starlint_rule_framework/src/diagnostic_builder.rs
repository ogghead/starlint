//! Ergonomic builder for constructing [`Diagnostic`] objects.
//!
//! Mirrors the [`FixBuilder`](crate::fix_builder::FixBuilder) API pattern,
//! reducing boilerplate when rules emit diagnostics.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Label, Severity, Span};

/// Builder for constructing a [`Diagnostic`].
///
/// # Example
///
/// ```ignore
/// let diag = DiagnosticBuilder::new("no-debugger", "Unexpected `debugger` statement", span)
///     .severity(Severity::Error)
///     .help("Remove the `debugger` statement")
///     .build();
/// ctx.report(diag);
/// ```
pub struct DiagnosticBuilder {
    /// Rule name.
    rule_name: String,
    /// Diagnostic message.
    message: String,
    /// Primary span.
    span: Span,
    /// Severity level.
    severity: Severity,
    /// Optional help text.
    help: Option<String>,
    /// Optional fix.
    fix: Option<Fix>,
    /// Additional labels.
    labels: Vec<Label>,
}

impl DiagnosticBuilder {
    /// Create a new builder with the required fields.
    ///
    /// Defaults to [`Severity::Error`] with no help, fix, or labels.
    #[must_use]
    pub fn new(rule_name: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            rule_name: rule_name.into(),
            message: message.into(),
            span,
            severity: Severity::Error,
            help: None,
            fix: None,
            labels: Vec::new(),
        }
    }

    /// Set the severity level.
    #[must_use]
    pub const fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the help text.
    #[must_use]
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Set the auto-fix.
    #[must_use]
    pub fn fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Set the auto-fix from an `Option<Fix>` (e.g. from [`FixBuilder::build()`]).
    #[must_use]
    pub fn maybe_fix(mut self, fix: Option<Fix>) -> Self {
        self.fix = fix;
        self
    }

    /// Add a labeled span for additional context.
    #[must_use]
    pub fn label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    /// Build the [`Diagnostic`].
    #[must_use]
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            rule_name: self.rule_name,
            message: self.message,
            span: self.span,
            severity: self.severity,
            help: self.help,
            fix: self.fix,
            labels: self.labels,
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use starlint_plugin_sdk::diagnostic::Edit;
    use starlint_plugin_sdk::rule::FixKind;

    #[test]
    fn test_basic_builder() {
        let diag = DiagnosticBuilder::new("test/rule", "bad code", Span::new(0, 5)).build();
        assert_eq!(diag.rule_name, "test/rule", "rule name should match");
        assert_eq!(diag.message, "bad code", "message should match");
        assert_eq!(diag.severity, Severity::Error, "default severity is Error");
        assert!(diag.help.is_none(), "help should be None by default");
        assert!(diag.fix.is_none(), "fix should be None by default");
        assert!(diag.labels.is_empty(), "labels should be empty by default");
    }

    #[test]
    fn test_builder_with_severity() {
        let diag = DiagnosticBuilder::new("test/rule", "msg", Span::new(0, 1))
            .severity(Severity::Warning)
            .build();
        assert_eq!(
            diag.severity,
            Severity::Warning,
            "severity should be Warning"
        );
    }

    #[test]
    fn test_builder_with_help() {
        let diag = DiagnosticBuilder::new("test/rule", "msg", Span::new(0, 1))
            .help("fix it")
            .build();
        assert_eq!(diag.help.as_deref(), Some("fix it"), "help should match");
    }

    #[test]
    fn test_builder_with_fix() {
        let fix = Fix {
            kind: FixKind::SafeFix,
            message: "fix".to_owned(),
            edits: vec![Edit {
                span: Span::new(0, 1),
                replacement: "x".to_owned(),
            }],
            is_snippet: false,
        };
        let diag = DiagnosticBuilder::new("test/rule", "msg", Span::new(0, 1))
            .fix(fix)
            .build();
        assert!(diag.fix.is_some(), "fix should be present");
    }

    #[test]
    fn test_builder_with_maybe_fix_none() {
        let diag = DiagnosticBuilder::new("test/rule", "msg", Span::new(0, 1))
            .maybe_fix(None)
            .build();
        assert!(diag.fix.is_none(), "fix should be None");
    }

    #[test]
    fn test_builder_with_label() {
        let diag = DiagnosticBuilder::new("test/rule", "msg", Span::new(0, 5))
            .label(Span::new(10, 15), "related")
            .build();
        assert_eq!(diag.labels.len(), 1, "should have one label");
        assert_eq!(
            diag.labels.first().map(|l| l.message.as_str()),
            Some("related"),
            "label message should match"
        );
    }

    #[test]
    fn test_builder_full_chain() {
        let diag = DiagnosticBuilder::new("no-debugger", "Unexpected debugger", Span::new(0, 8))
            .severity(Severity::Error)
            .help("Remove the debugger statement")
            .label(Span::new(0, 8), "here")
            .build();
        assert_eq!(diag.rule_name, "no-debugger", "rule name");
        assert_eq!(diag.severity, Severity::Error, "severity");
        assert!(diag.help.is_some(), "help present");
        assert_eq!(diag.labels.len(), 1, "one label");
    }
}
