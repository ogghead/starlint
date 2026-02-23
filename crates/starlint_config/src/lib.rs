//! Configuration file loading and resolution for starlint.
//!
//! Supports `starlint.toml` config files with rule severity overrides,
//! plugin declarations, and file-pattern-based overrides.

#[allow(unused_assignments)] // False positive from thiserror 2.x macro-generated Display impls
pub mod error;
pub mod resolve;

pub use error::ConfigError;

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// General settings.
    #[serde(default)]
    pub settings: Settings,

    /// Unified plugin declarations.
    ///
    /// Each entry maps a plugin name to its configuration. Plugins can be
    /// toggled with a simple boolean or configured with a path and options:
    ///
    /// ```toml
    /// [plugins]
    /// core = true
    /// react = true
    /// typescript = false
    /// custom = { path = "./plugins/custom.wasm" }
    /// ```
    ///
    /// When the `[plugins]` section is absent or empty, all built-in plugins
    /// are enabled by default.
    #[serde(default)]
    pub plugins: HashMap<String, PluginEntry>,

    /// Rule configurations: rule name -> severity or detailed config.
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,

    /// File-pattern overrides.
    ///
    /// Override blocks match files by glob pattern and adjust rule severity
    /// for matching files. Multiple matching blocks merge in order (later wins).
    #[serde(default)]
    pub overrides: Vec<Override>,
}

/// General settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Number of threads (0 = auto-detect).
    #[serde(default)]
    pub threads: usize,
}

/// Configuration for a single plugin.
///
/// Supports two forms:
/// - Simple toggle: `react = true` / `react = false`
/// - Detailed: `custom = { path = "./plugin.wasm", enabled = true }`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginEntry {
    /// Simple boolean toggle.
    Toggle(bool),
    /// Detailed plugin configuration with optional path.
    Detailed(PluginDetail),
}

impl PluginEntry {
    /// Whether this plugin is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        match self {
            Self::Toggle(enabled) => *enabled,
            Self::Detailed(detail) => detail.enabled,
        }
    }

    /// Optional path to a WASM plugin file.
    #[must_use]
    pub const fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Toggle(_) => None,
            Self::Detailed(detail) => detail.path.as_ref(),
        }
    }
}

/// Detailed plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDetail {
    /// Path to a WASM plugin file. When absent, the plugin is resolved
    /// from the built-in registry.
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Whether this plugin is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Default value for `enabled` field in `PluginDetail` (true).
const fn default_true() -> bool {
    true
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
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(
            result.is_ok(),
            "empty config should deserialize to defaults"
        );
    }

    #[test]
    fn test_config_deserialize_with_rules() {
        let toml_str = r#"
[rules]
"no-debugger" = "error"
"no-console" = "warn"
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_ok(), "config with rules should deserialize");
        if let Ok(cfg) = result {
            assert_eq!(cfg.rules.len(), 2, "should have two rules");
        }
    }

    #[test]
    fn test_config_deserialize_with_plugin_toggle() {
        let toml_str = r"
[plugins]
react = true
storybook = false
";
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(
            result.is_ok(),
            "config with plugin toggles should deserialize"
        );
        if let Ok(cfg) = result {
            assert_eq!(cfg.plugins.len(), 2);
            assert!(
                cfg.plugins
                    .get("react")
                    .is_some_and(PluginEntry::is_enabled)
            );
            assert!(
                !cfg.plugins
                    .get("storybook")
                    .is_some_and(PluginEntry::is_enabled)
            );
        }
    }

    #[test]
    fn test_config_deserialize_with_plugin_path() {
        let toml_str = r#"
[plugins]
custom = { path = "./plugins/custom.wasm" }
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_ok(), "config with plugin path should deserialize");
        if let Ok(cfg) = result {
            assert!(
                cfg.plugins
                    .get("custom")
                    .is_some_and(PluginEntry::is_enabled),
                "plugin with path should default to enabled"
            );
            assert!(
                cfg.plugins
                    .get("custom")
                    .and_then(PluginEntry::path)
                    .is_some(),
                "plugin should have path"
            );
        }
    }

    #[test]
    fn test_config_deserialize_with_plugin_disabled() {
        let toml_str = r#"
[plugins]
custom = { path = "./plugins/custom.wasm", enabled = false }
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(
            result.is_ok(),
            "config with disabled plugin should deserialize"
        );
        if let Ok(cfg) = result {
            assert!(
                !cfg.plugins
                    .get("custom")
                    .is_some_and(PluginEntry::is_enabled),
                "plugin should be disabled"
            );
        }
    }

    #[test]
    fn test_config_deserialize_with_overrides() {
        let toml_str = r#"
[[overrides]]
files = ["**/*.stories.tsx"]

[overrides.rules]
"storybook/default-exports" = "error"
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_ok(), "config with overrides should deserialize");
    }

    #[test]
    fn test_config_plugins_default_empty() {
        let toml_str = "";
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_ok(), "should parse empty config");
        if let Ok(cfg) = result {
            assert!(cfg.plugins.is_empty(), "plugins should default to empty");
        }
    }

    #[test]
    fn test_config_mixed_plugins() {
        let toml_str = r#"
[plugins]
core = true
react = true
typescript = false
custom = { path = "./plugins/custom.wasm" }
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_ok(), "mixed plugin config should deserialize");
        if let Ok(cfg) = result {
            assert_eq!(cfg.plugins.len(), 4);
            assert!(cfg.plugins.get("core").is_some_and(PluginEntry::is_enabled));
            assert!(
                cfg.plugins
                    .get("react")
                    .is_some_and(PluginEntry::is_enabled)
            );
            assert!(
                !cfg.plugins
                    .get("typescript")
                    .is_some_and(PluginEntry::is_enabled)
            );
            assert!(
                cfg.plugins
                    .get("custom")
                    .is_some_and(PluginEntry::is_enabled)
            );
        }
    }
}
