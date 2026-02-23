//! Core error types for the linting engine.

use miette::Diagnostic;
use thiserror::Error;

use starlint_plugin_sdk::diagnostic::{Diagnostic as LintDiagnostic, Severity, Span};

/// Errors that can occur during linting.
///
/// Used by the parser and engine to produce structured errors that can be
/// converted to user-facing diagnostics via [`LintError::into_diagnostic`].
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum LintError {
    /// A file could not be read.
    #[error("failed to read file: {path}")]
    #[diagnostic(code(starlint::io), help("Check that the file exists and is readable"))]
    FileRead {
        /// Path of the file that could not be read.
        path: String,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// A file could not be parsed.
    #[error("parse error in {path}")]
    #[diagnostic(
        code(starlint::parse),
        help("Check that the file is valid JavaScript/TypeScript")
    )]
    Parse {
        /// Path of the file that failed to parse.
        path: String,
    },
}

impl LintError {
    /// Convert this error into a synthetic lint diagnostic.
    ///
    /// Produces a `Diagnostic` with a `starlint/io-error` or `starlint/parse-error`
    /// rule name, suitable for inclusion in file-level diagnostic output.
    #[must_use]
    pub fn into_diagnostic(self) -> LintDiagnostic {
        match &self {
            Self::FileRead { path: _, source } => LintDiagnostic {
                rule_name: "starlint/io-error".to_owned(),
                message: format!("Failed to read file: {source}"),
                span: Span::new(0, 0),
                severity: Severity::Error,
                help: Some("Check that the file exists and is readable".to_owned()),
                fix: None,
                labels: vec![],
            },
            Self::Parse { path } => LintDiagnostic {
                rule_name: "starlint/parse-error".to_owned(),
                message: format!("Failed to parse file: unsupported file type {path}"),
                span: Span::new(0, 0),
                severity: Severity::Error,
                help: Some("Check that the file is valid JavaScript/TypeScript".to_owned()),
                fix: None,
                labels: vec![],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_lint_error_file_read_display() {
        let err = LintError::FileRead {
            path: "foo.ts".to_owned(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        assert_eq!(
            err.to_string(),
            "failed to read file: foo.ts",
            "file read error format must be stable"
        );
    }

    #[test]
    fn test_lint_error_parse_display() {
        let err = LintError::Parse {
            path: "bar.tsx".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "parse error in bar.tsx",
            "parse error format must be stable"
        );
    }

    #[test]
    fn test_file_read_into_diagnostic() {
        let err = LintError::FileRead {
            path: "test.js".to_owned(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let diag = err.into_diagnostic();
        assert_eq!(diag.rule_name, "starlint/io-error");
        assert_eq!(diag.severity, Severity::Error);
        assert!(
            diag.message.contains("not found"),
            "diagnostic message should contain IO error"
        );
    }

    #[test]
    fn test_parse_into_diagnostic() {
        let err = LintError::Parse {
            path: "test.py".to_owned(),
        };
        let diag = err.into_diagnostic();
        assert_eq!(diag.rule_name, "starlint/parse-error");
        assert_eq!(diag.severity, Severity::Error);
        assert!(
            diag.message.contains("test.py"),
            "diagnostic message should contain file path"
        );
    }
}
