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
    /// Config files to extend. Paths are relative to the config file's directory.
    /// Built-in presets: `"starlint:recommended"`, `"starlint:strict"`.
    #[serde(default)]
    pub extends: Vec<String>,

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

impl Config {
    /// Merge a base config into this one. `self` (the child/local) takes priority.
    ///
    /// - `settings`: `self` wins if non-default (non-zero threads).
    /// - `plugins`: `self` entries override base entries by key.
    /// - `rules`: `self` entries override base entries by key.
    /// - `overrides`: base overrides come first, then `self`'s are appended.
    /// - `extends`: not merged (only the top-level `extends` is processed).
    pub fn merge_from(&mut self, base: &Self) {
        // Settings: keep self's value if non-zero, else use base.
        if self.settings.threads == 0 {
            self.settings.threads = base.settings.threads;
        }

        // Plugins: base entries fill in gaps; self's entries take priority.
        for (name, entry) in &base.plugins {
            self.plugins
                .entry(name.clone())
                .or_insert_with(|| entry.clone());
        }

        // Rules: base entries fill in gaps; self's entries take priority.
        for (name, rule) in &base.rules {
            self.rules
                .entry(name.clone())
                .or_insert_with(|| rule.clone());
        }

        // Overrides: base overrides come first, then self's.
        let mut merged_overrides = base.overrides.clone();
        merged_overrides.append(&mut self.overrides);
        self.overrides = merged_overrides;
    }
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
    fn test_config_deserialize_with_extends() {
        let toml_str = r#"
extends = ["./base.toml", "starlint:recommended"]

[rules]
"no-debugger" = "error"
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_ok(), "config with extends should deserialize");
        if let Ok(cfg) = result {
            assert_eq!(cfg.extends.len(), 2, "should have two extends entries");
            assert_eq!(cfg.extends.first().map(String::as_str), Some("./base.toml"));
            assert_eq!(
                cfg.extends.get(1).map(String::as_str),
                Some("starlint:recommended")
            );
        }
    }

    #[test]
    fn test_config_merge_from_rules() {
        let mut local = Config::default();
        local.rules.insert(
            "no-debugger".to_owned(),
            RuleConfig::Severity("error".to_owned()),
        );
        local.rules.insert(
            "no-console".to_owned(),
            RuleConfig::Severity("warn".to_owned()),
        );

        let mut base = Config::default();
        base.rules.insert(
            "no-console".to_owned(),
            RuleConfig::Severity("error".to_owned()),
        );
        base.rules.insert(
            "no-eval".to_owned(),
            RuleConfig::Severity("error".to_owned()),
        );

        local.merge_from(&base);

        assert_eq!(local.rules.len(), 3, "should have three rules after merge");
        // local's no-console (warn) should win over base's (error)
        assert!(matches!(
            local.rules.get("no-console"),
            Some(RuleConfig::Severity(s)) if s == "warn"
        ));
        // base's no-eval should be present
        assert!(local.rules.contains_key("no-eval"));
    }

    #[test]
    fn test_config_merge_from_plugins() {
        let mut local = Config::default();
        local
            .plugins
            .insert("react".to_owned(), PluginEntry::Toggle(false));

        let mut base = Config::default();
        base.plugins
            .insert("react".to_owned(), PluginEntry::Toggle(true));
        base.plugins
            .insert("core".to_owned(), PluginEntry::Toggle(true));

        local.merge_from(&base);

        assert_eq!(
            local.plugins.len(),
            2,
            "should have two plugins after merge"
        );
        // local's react=false should win
        assert!(
            !local
                .plugins
                .get("react")
                .is_some_and(PluginEntry::is_enabled)
        );
        // base's core=true should be inherited
        assert!(
            local
                .plugins
                .get("core")
                .is_some_and(PluginEntry::is_enabled)
        );
    }

    #[test]
    fn test_config_merge_from_overrides() {
        let mut local = Config::default();
        local.overrides.push(Override {
            files: vec!["**/*.test.ts".to_owned()],
            rules: HashMap::new(),
        });

        let mut base = Config::default();
        base.overrides.push(Override {
            files: vec!["**/*.stories.tsx".to_owned()],
            rules: HashMap::new(),
        });

        local.merge_from(&base);

        assert_eq!(
            local.overrides.len(),
            2,
            "should have two overrides after merge"
        );
        // Base overrides come first.
        assert_eq!(
            local
                .overrides
                .first()
                .and_then(|o| o.files.first())
                .map(String::as_str),
            Some("**/*.stories.tsx"),
            "base override should come first"
        );
        assert_eq!(
            local
                .overrides
                .get(1)
                .and_then(|o| o.files.first())
                .map(String::as_str),
            Some("**/*.test.ts"),
            "local override should come second"
        );
    }

    #[test]
    fn test_config_merge_from_settings() {
        let mut local = Config::default();
        // threads == 0 (default)

        let mut base = Config::default();
        base.settings.threads = 4;

        local.merge_from(&base);
        assert_eq!(
            local.settings.threads, 4,
            "base threads should be used when local is default"
        );

        // Now test that local non-zero wins
        let mut local2 = Config::default();
        local2.settings.threads = 8;
        local2.merge_from(&base);
        assert_eq!(
            local2.settings.threads, 8,
            "local threads should win when non-zero"
        );
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
