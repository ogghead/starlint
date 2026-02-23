//! Configuration file loading and resolution for starlint.
//!
//! Supports `starlint.toml` config files with rule severity overrides,
//! plugin declarations, and file-pattern-based overrides.

pub mod resolve;

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// General settings.
    #[serde(default)]
    pub settings: Settings,

    /// Plugin declarations.
    #[serde(default)]
    pub plugins: Vec<PluginDeclaration>,

    /// Rule configurations: rule name → severity or detailed config.
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,

    /// File-pattern overrides.
    #[serde(default)]
    pub overrides: Vec<Override>,
}

/// General settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Number of threads (0 = auto-detect).
    #[serde(default)]
    pub threads: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self { threads: 0 }
    }
}

/// A plugin declaration in config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDeclaration {
    /// Plugin name (used as prefix for its rules).
    pub name: String,
    /// Path to the WASM plugin file.
    pub path: PathBuf,
}

/// Configuration for a single rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    /// Simple severity string: "error", "warn", "off".
    Severity(String),
    /// Detailed config with severity and rule-specific options.
    Detailed(DetailedRuleConfig),
}

/// Detailed rule configuration with options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedRuleConfig {
    /// Severity level.
    pub severity: String,
    /// Rule-specific options (varies per rule).
    #[serde(flatten)]
    pub options: HashMap<String, serde_json::Value>,
}

/// A file-pattern override block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Override {
    /// Glob patterns matching files this override applies to.
    pub files: Vec<String>,
    /// Rule overrides for matching files.
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_config_deserialize_minimal() {
        let toml_str = "";
        let config: Result<Config, _> = toml::from_str(toml_str);
        assert!(config.is_ok(), "empty config should deserialize to defaults");
    }

    #[test]
    fn test_config_deserialize_with_rules() {
        let toml_str = r#"
[rules]
"no-debugger" = "error"
"no-console" = "warn"
"#;
        let config: Result<Config, _> = toml::from_str(toml_str);
        assert!(config.is_ok(), "config with rules should deserialize");
        let config = config.ok();
        assert!(config.is_some(), "should have config");
        if let Some(cfg) = config {
            assert_eq!(cfg.rules.len(), 2, "should have two rules");
        }
    }

    #[test]
    fn test_config_deserialize_with_plugin() {
        let toml_str = r#"
[[plugins]]
name = "storybook"
path = "./plugins/storybook.wasm"
"#;
        let config: Result<Config, _> = toml::from_str(toml_str);
        assert!(config.is_ok(), "config with plugin should deserialize");
    }

    #[test]
    fn test_config_deserialize_with_overrides() {
        let toml_str = r#"
[[overrides]]
files = ["**/*.stories.tsx"]

[overrides.rules]
"storybook/default-exports" = "error"
"#;
        let config: Result<Config, _> = toml::from_str(toml_str);
        assert!(config.is_ok(), "config with overrides should deserialize");
    }
}
