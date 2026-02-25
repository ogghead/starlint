//! CLI-level error types.

use miette::Diagnostic;
use thiserror::Error;

/// Top-level CLI errors.
///
/// Note: These variants are defined for structured error reporting but are
/// not yet wired into all production code paths. Some error sites currently
/// use `miette!()` or `eprintln!` directly. TODO: Migrate those sites to
/// use `CliError` for consistent error handling.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum CliError {
    /// Configuration file error.
    #[error("configuration error: {0}")]
    #[diagnostic(code(starlint::config), help("Check your starlint.toml configuration"))]
    Config(String),

    /// No files found to lint.
    #[error("no lintable files found in the given paths")]
    #[diagnostic(
        code(starlint::no_files),
        help("Check that the paths contain JS/TS files")
    )]
    NoFiles,
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_cli_error_display() {
        let err = CliError::Config("bad config".to_owned());
        assert_eq!(
            err.to_string(),
            "configuration error: bad config",
            "config error format must be stable"
        );
    }
}
