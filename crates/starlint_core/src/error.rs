//! Core error types for the linting engine.

use miette::Diagnostic;
use thiserror::Error;

/// Errors that can occur during linting.
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

    /// Semantic analysis failed.
    #[error("semantic analysis error in {path}")]
    #[diagnostic(code(starlint::semantic))]
    Semantic {
        /// Path of the file.
        path: String,
    },
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
}
