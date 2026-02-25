//! CLI-level error types.

use miette::Diagnostic;
use thiserror::Error;

/// Top-level CLI errors.
///
/// Wraps domain errors from lower crates (`ConfigError`) and adds CLI-specific
/// variants for init and runtime failures.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum CliError {
    /// Configuration file error (delegates to `ConfigError`).
    #[error(transparent)]
    #[diagnostic(transparent)]
    Config(#[from] starlint_config::ConfigError),

    /// Failed to initialize default config file.
    #[error("failed to initialize config: {0}")]
    #[diagnostic(
        code(starlint::init),
        help("Check file permissions in the current directory")
    )]
    Init(String),

    /// Tokio runtime creation failed.
    #[error("failed to create async runtime: {0}")]
    #[diagnostic(code(starlint::runtime))]
    Runtime(String),
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_cli_error_config_display() {
        let inner = starlint_config::ConfigError::ParseFailed {
            path: "starlint.toml".to_owned(),
            reason: "bad toml".to_owned(),
        };
        let err = CliError::Config(inner);
        assert_eq!(
            err.to_string(),
            "failed to parse config file starlint.toml: bad toml",
            "config error should delegate to ConfigError display"
        );
    }

    #[test]
    fn test_cli_error_init_display() {
        let err = CliError::Init("permission denied".to_owned());
        assert_eq!(
            err.to_string(),
            "failed to initialize config: permission denied",
            "init error format must be stable"
        );
    }

    #[test]
    fn test_cli_error_runtime_display() {
        let err = CliError::Runtime("could not spawn threads".to_owned());
        assert_eq!(
            err.to_string(),
            "failed to create async runtime: could not spawn threads",
            "runtime error format must be stable"
        );
    }
}
