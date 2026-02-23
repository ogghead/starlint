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

    #[test]
    fn test_load_config_nonexistent_file() {
        let result = load_config(Path::new("/nonexistent/starlint.toml"));
        assert!(result.is_err(), "loading a nonexistent file should fail");
        assert!(
            matches!(result, Err(ConfigError::ReadFailed { .. })),
            "nonexistent file should produce ReadFailed error"
        );
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    fn test_load_config_invalid_toml() {
        let dir = std::env::temp_dir().join("starlint_test_invalid_toml");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join("starlint.toml");
        let _ = std::fs::write(&config_path, "not valid {{{{ toml content %%%");

        let result = load_config(&config_path);

        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_err(), "invalid TOML content should fail to parse");
        assert!(
            matches!(result, Err(ConfigError::ParseFailed { .. })),
            "invalid TOML should produce ParseFailed error"
        );
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    fn test_load_config_valid_toml() {
        let dir = std::env::temp_dir().join("starlint_test_valid_toml");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join("starlint.toml");
        let _ = std::fs::write(
            &config_path,
            r#"
[rules]
"no-debugger" = "error"
"#,
        );

        let result = load_config(&config_path);

        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok(), "valid TOML config should load successfully");
        if let Ok(cfg) = result {
            assert_eq!(
                cfg.rules.len(),
                1,
                "loaded config should contain the one rule declared in the TOML"
            );
        }
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    fn test_find_config_file_in_current_dir() {
        let dir = std::env::temp_dir().join("starlint_test_find_current");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join("starlint.toml");
        let _ = std::fs::write(&config_path, "");

        let result = find_config_file(&dir);

        let _ = std::fs::remove_dir_all(&dir);

        assert!(
            result.is_some(),
            "should find starlint.toml in the start directory"
        );
        assert_eq!(
            result.as_deref(),
            Some(config_path.as_path()),
            "returned path should match the config file in the directory"
        );
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    fn test_find_config_file_in_parent_dir() {
        let parent = std::env::temp_dir().join("starlint_test_find_parent");
        let child = parent.join("subdir");
        let _ = std::fs::create_dir_all(&child);
        let config_path = parent.join("starlint.toml");
        let _ = std::fs::write(&config_path, "");

        let result = find_config_file(&child);

        let _ = std::fs::remove_dir_all(&parent);

        assert!(
            result.is_some(),
            "should find starlint.toml by walking up from subdirectory"
        );
        assert_eq!(
            result.as_deref(),
            Some(config_path.as_path()),
            "returned path should point to the config file in the parent directory"
        );
    }
}
