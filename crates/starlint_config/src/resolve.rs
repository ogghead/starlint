//! Config file discovery and resolution.
//!
//! Walks from the target directory upward to find a `starlint.toml` config file.
//! Merges base rules with matching override blocks.

use std::collections::HashSet;
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

/// Load and parse a config file, resolving any `extends` chains.
///
/// # Errors
///
/// Returns `ConfigError::ReadFailed` if the file cannot be read,
/// `ConfigError::ParseFailed` if it is not valid TOML, or
/// `ConfigError::CircularExtend` if a circular extends chain is detected.
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let mut visited = HashSet::new();
    load_config_inner(path, &mut visited)
}

/// Inner config loader that tracks visited paths for cycle detection.
fn load_config_inner(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|err| ConfigError::ReadFailed {
        path: path.display().to_string(),
        source: err,
    })?;

    let mut config: Config = toml::from_str(&content).map_err(|err| ConfigError::ParseFailed {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;

    if !config.extends.is_empty() {
        let config_dir = path.parent().unwrap_or_else(|| Path::new("."));
        config = resolve_extends(config, config_dir, visited)?;
    }

    Ok(config)
}

/// Resolve the `extends` chain.
///
/// Loads each extended config (which may itself have extends), merges them
/// in order, then merges the local config on top. Detects circular extends
/// by tracking visited paths.
///
/// # Errors
///
/// Returns `ConfigError::CircularExtend` if a cycle is found, or propagates
/// errors from loading extended config files.
fn resolve_extends(
    config: Config,
    config_dir: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Config, ConfigError> {
    let mut base = Config::default();

    for extend_ref in &config.extends {
        let extended = resolve_single_extend(extend_ref, config_dir, visited)?;
        base.merge_from(&extended);
    }

    // Merge local config on top of the resolved base — local takes priority.
    let mut local = config;
    local.merge_from(&base);
    Ok(local)
}

/// Resolve a single `extends` reference, which may be a built-in preset
/// (e.g., `"starlint:recommended"`) or a relative file path.
///
/// # Errors
///
/// Returns `ConfigError::CircularExtend` if the path was already visited,
/// or propagates errors from loading the extended config file.
fn resolve_single_extend(
    extend_ref: &str,
    config_dir: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Config, ConfigError> {
    // Handle built-in presets.
    if let Some(preset) = extend_ref.strip_prefix("starlint:") {
        return Ok(builtin_preset(preset));
    }

    // Resolve relative path.
    let extend_path = config_dir.join(extend_ref);
    let canonical = extend_path
        .canonicalize()
        .map_err(|err| ConfigError::ReadFailed {
            path: extend_path.display().to_string(),
            source: err,
        })?;

    if !visited.insert(canonical.clone()) {
        return Err(ConfigError::CircularExtend {
            path: canonical.display().to_string(),
        });
    }

    load_config_inner(&canonical, visited)
}

/// Return a built-in preset config.
///
/// Currently supported presets:
/// - `"recommended"`: the default config (all defaults).
/// - `"strict"`: all defaults (placeholder for future strict rules).
///
/// Unknown presets log a warning and return defaults.
#[must_use]
fn builtin_preset(name: &str) -> Config {
    match name {
        "recommended" => Config::default(),
        "strict" => {
            // Placeholder: strict preset returns defaults for now.
            Config::default()
        }
        _ => {
            tracing::warn!("unknown preset: starlint:{name}, using defaults");
            Config::default()
        }
    }
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
    fn test_load_config_with_extends() {
        let dir = std::env::temp_dir().join("starlint_test_extends");
        let _ = std::fs::create_dir_all(&dir);

        // Write base config.
        let base_path = dir.join("base.toml");
        let _ = std::fs::write(
            &base_path,
            r#"
[rules]
"no-eval" = "error"
"no-console" = "error"
"#,
        );

        // Write child config that extends base.
        let child_path = dir.join("starlint.toml");
        let _ = std::fs::write(
            &child_path,
            r#"
extends = ["./base.toml"]

[rules]
"no-console" = "warn"
"no-debugger" = "error"
"#,
        );

        let result = load_config(&child_path);

        let _ = std::fs::remove_dir_all(&dir);

        assert!(
            result.is_ok(),
            "config with extends should load: {result:?}"
        );
        if let Ok(cfg) = result {
            assert_eq!(cfg.rules.len(), 3, "should have three rules after merge");
            // Child's no-console=warn should win over base's no-console=error.
            assert!(matches!(
                cfg.rules.get("no-console"),
                Some(crate::RuleConfig::Severity(s)) if s == "warn"
            ));
            // Base's no-eval should be inherited.
            assert!(cfg.rules.contains_key("no-eval"));
            // Child's no-debugger should be present.
            assert!(cfg.rules.contains_key("no-debugger"));
        }
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    fn test_circular_extend_detected() {
        let dir = std::env::temp_dir().join("starlint_test_circular");
        let _ = std::fs::create_dir_all(&dir);

        // a.toml extends b.toml, b.toml extends a.toml.
        let a_path = dir.join("a.toml");
        let b_path = dir.join("b.toml");
        let _ = std::fs::write(&a_path, "extends = [\"./b.toml\"]\n");
        let _ = std::fs::write(&b_path, "extends = [\"./a.toml\"]\n");

        let result = load_config(&a_path);

        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_err(), "circular extends should produce an error");
        assert!(
            matches!(result, Err(ConfigError::CircularExtend { .. })),
            "should be a CircularExtend error"
        );
    }

    #[test]
    fn test_builtin_preset_recommended() {
        let preset = builtin_preset("recommended");
        // The recommended preset should be equivalent to default config.
        assert!(
            preset.rules.is_empty(),
            "recommended preset should have no explicit rules"
        );
        assert!(
            preset.plugins.is_empty(),
            "recommended preset should have no explicit plugins"
        );
    }

    #[test]
    fn test_builtin_preset_strict() {
        let preset = builtin_preset("strict");
        // Strict is currently a placeholder returning defaults.
        assert!(
            preset.rules.is_empty(),
            "strict preset placeholder should have no explicit rules"
        );
    }

    #[test]
    fn test_builtin_preset_unknown() {
        let preset = builtin_preset("nonexistent");
        assert!(
            preset.rules.is_empty(),
            "unknown preset should fall back to defaults"
        );
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    fn test_load_config_with_builtin_extends() {
        let dir = std::env::temp_dir().join("starlint_test_builtin_extends");
        let _ = std::fs::create_dir_all(&dir);

        let config_path = dir.join("starlint.toml");
        let _ = std::fs::write(
            &config_path,
            r#"
extends = ["starlint:recommended"]

[rules]
"no-debugger" = "error"
"#,
        );

        let result = load_config(&config_path);

        let _ = std::fs::remove_dir_all(&dir);

        assert!(
            result.is_ok(),
            "config extending a builtin preset should load"
        );
        if let Ok(cfg) = result {
            assert!(cfg.rules.contains_key("no-debugger"));
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
