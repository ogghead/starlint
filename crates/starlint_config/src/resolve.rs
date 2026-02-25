//! Config file discovery and resolution.
//!
//! Walks from the target directory upward to find a `starlint.toml` config file.
//! Merges base rules with matching override blocks.

use std::path::{Path, PathBuf};

use crate::Config;
use crate::error::ConfigError;

/// Config file names to look for, in priority order.
const CONFIG_FILE_NAMES: &[&str] = &["starlint.toml"];

/// Discover a config file by walking from `start_dir` up to the filesystem root.
///
/// Returns `None` if no config file is found (which is valid — use defaults).
#[must_use]
pub fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        for name in CONFIG_FILE_NAMES {
            let candidate = current.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        if !current.pop() {
            break;
        }
    }

    None
}

/// Load and parse a config file.
///
/// # Errors
///
/// Returns `ConfigError::ReadFailed` if the file cannot be read, or
/// `ConfigError::ParseFailed` if it is not valid TOML.
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|err| ConfigError::ReadFailed {
        path: path.display().to_string(),
        source: err,
    })?;

    let config: Config = toml::from_str(&content).map_err(|err| ConfigError::ParseFailed {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;

    Ok(config)
}

/// Load config from a directory, or return defaults if no config file exists.
///
/// # Errors
///
/// Returns `ConfigError` if a config file is found but cannot be read or parsed.
pub fn resolve_config(start_dir: &Path) -> Result<Config, ConfigError> {
    if let Some(path) = find_config_file(start_dir) {
        tracing::info!("using config: {}", path.display());
        load_config(&path)
    } else {
        tracing::debug!("no config file found, using defaults");
        Ok(Config::default())
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_find_config_file_nonexistent() {
        let result = find_config_file(Path::new("/nonexistent/path"));
        assert!(result.is_none(), "should return None for nonexistent path");
    }

    #[test]
    fn test_resolve_config_defaults() {
        let result = resolve_config(Path::new("/tmp"));
        assert!(
            result.is_ok(),
            "should return defaults when no config file exists"
        );
    }
}
