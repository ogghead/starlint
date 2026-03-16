//! Configuration error types.

use miette::Diagnostic;
use thiserror::Error;

/// Errors from configuration file loading and parsing.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ConfigError {
    /// Config file could not be read.
    #[error("failed to read config file {path}: {source}")]
    #[diagnostic(
        code(starlint::config::read),
        help("Check that the config file exists and is readable")
    )]
    ReadFailed {
        /// Path of the config file.
        path: String,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Config file could not be parsed as TOML.
    #[error("failed to parse config file {path}: {reason}")]
    #[diagnostic(
        code(starlint::config::parse),
        help("Check that the config file is valid TOML")
    )]
    ParseFailed {
        /// Path of the config file.
        path: String,
        /// Parse error details.
        reason: String,
    },

    /// Circular `extends` chain detected.
    #[error("circular extends detected: {path} was already visited")]
    #[diagnostic(
        code(starlint::config::circular_extend),
        help("Remove the circular reference in your extends chain")
    )]
    CircularExtend {
        /// Path that was visited more than once.
        path: String,
    },
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_config_error_read_failed_display() {
        let err = ConfigError::ReadFailed {
            path: "starlint.toml".to_owned(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        assert_eq!(
            err.to_string(),
            "failed to read config file starlint.toml: not found",
            "read error format must be stable"
        );
    }

    #[test]
    fn test_config_error_circular_extend_display() {
        let err = ConfigError::CircularExtend {
            path: "/configs/a.toml".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "circular extends detected: /configs/a.toml was already visited",
            "circular extend error format must be stable"
        );
    }

    #[test]
    fn test_config_error_parse_failed_display() {
        let err = ConfigError::ParseFailed {
            path: "starlint.toml".to_owned(),
            reason: "expected table header".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "failed to parse config file starlint.toml: expected table header",
            "parse error format must be stable"
        );
    }
}
