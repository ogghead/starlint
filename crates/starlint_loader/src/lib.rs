//! Unified plugin loader for starlint.
//!
//! Resolves plugin names from config into [`Plugin`] trait objects,
//! handling both native Rust rule bundles and WASM plugins through
//! a single code path.

use std::collections::{HashMap, HashSet};

use starlint_config::{Config, RuleConfig};
use starlint_plugin_sdk::diagnostic::{Severity, parse_severity};
use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

#[cfg(feature = "wasm")]
use starlint_wasm_host::runtime::{ResourceLimits, WasmPluginHost};

/// Result of loading and configuring plugins.
pub struct LoadedPlugins {
    /// All loaded plugins (native + WASM), ready for `LintSession::new()`.
    pub plugins: Vec<Box<dyn Plugin>>,
    /// Severity overrides from `[rules]` config.
    pub severity_overrides: HashMap<String, Severity>,
    /// Rules loaded but disabled by default (only active via file-pattern overrides).
    pub disabled_rules: HashSet<String>,
}

/// A registered native plugin: name + factory function.
struct NativePlugin {
    /// Plugin name (e.g., "core", "react", "typescript").
    name: &'static str,
    /// Factory function returning all rules in this bundle.
    factory: fn() -> Vec<Box<dyn LintRule>>,
}

/// All built-in native plugin bundles.
///
/// Each bundle corresponds to a named plugin in the config:
/// ```toml
/// [plugins]
/// core = true
/// react = true
/// ```
fn native_plugin_registry() -> Vec<NativePlugin> {
    vec![
        #[cfg(feature = "plugin-core")]
        NativePlugin {
            name: "core",
            factory: starlint_plugin_core::all_rules,
        },
        #[cfg(feature = "plugin-react")]
        NativePlugin {
            name: "react",
            factory: starlint_plugin_react::all_rules,
        },
        #[cfg(feature = "plugin-typescript")]
        NativePlugin {
            name: "typescript",
            factory: starlint_plugin_typescript::all_rules,
        },
        #[cfg(feature = "plugin-testing")]
        NativePlugin {
            name: "testing",
            factory: starlint_plugin_testing::all_rules,
        },
        #[cfg(feature = "plugin-modules")]
        NativePlugin {
            name: "modules",
            factory: starlint_plugin_modules::all_rules,
        },
        #[cfg(feature = "plugin-nextjs")]
        NativePlugin {
            name: "nextjs",
            factory: starlint_plugin_nextjs::all_rules,
        },
        #[cfg(feature = "plugin-vue")]
        NativePlugin {
            name: "vue",
            factory: starlint_plugin_vue::all_rules,
        },
        #[cfg(feature = "plugin-jsdoc")]
        NativePlugin {
            name: "jsdoc",
            factory: starlint_plugin_jsdoc::all_rules,
        },
        #[cfg(feature = "plugin-storybook")]
        NativePlugin {
            name: "storybook",
            factory: starlint_plugin_storybook::all_rules,
        },
    ]
}

/// Return all built-in lint rules from all native plugin crates.
#[must_use]
fn all_lint_rules() -> Vec<Box<dyn LintRule>> {
    native_plugin_registry()
        .into_iter()
        .flat_map(|np| (np.factory)())
        .collect()
}

/// Return metadata for all built-in rules.
#[must_use]
pub fn all_rule_metas() -> Vec<starlint_plugin_sdk::rule::RuleMeta> {
    all_lint_rules().iter().map(|r| r.meta()).collect()
}

/// Load all plugins from config.
///
/// When the `[plugins]` section is absent or empty, all built-in native
/// plugins are enabled by default. When plugins are listed explicitly,
/// only those plugins are loaded.
///
/// Resolution order for each plugin name:
/// 1. If `path` specified → external WASM from disk
/// 2. If name matches native registry → native `LintRulePlugin`
/// 3. Otherwise → warning logged, plugin skipped
#[allow(clippy::module_name_repetitions)]
pub fn load_plugins(config: &Config) -> LoadedPlugins {
    type RuleFactory = fn() -> Vec<Box<dyn LintRule>>;
    let registry: HashMap<&str, RuleFactory> = native_plugin_registry()
        .into_iter()
        .map(|np| (np.name, np.factory))
        .collect();

    let mut all_native_rules: Vec<Box<dyn LintRule>> = Vec::new();

    #[cfg(feature = "wasm")]
    let mut wasm_host: Option<WasmPluginHost> = None;

    if config.plugins.is_empty() {
        // No plugins section → enable all native plugins by default.
        for np in native_plugin_registry() {
            all_native_rules.extend((np.factory)());
        }
    } else {
        // Explicit plugin list — load only what's configured.
        for (name, entry) in &config.plugins {
            if !entry.is_enabled() {
                continue;
            }

            #[cfg(feature = "wasm")]
            if let Some(path) = entry.path() {
                // External WASM plugin from disk.
                let host = match wasm_host.as_mut() {
                    Some(h) => h,
                    None => match WasmPluginHost::new(ResourceLimits::default()) {
                        Ok(h) => wasm_host.insert(h),
                        Err(err) => {
                            tracing::warn!("failed to initialize WASM runtime: {err}");
                            continue;
                        }
                    },
                };
                if let Err(err) = host.load_plugin(path, "") {
                    tracing::warn!(
                        "failed to load WASM plugin `{name}` from {}: {err}",
                        path.display()
                    );
                }
                continue;
            }

            // Check native registry.
            if let Some(factory) = registry.get(name.as_str()) {
                all_native_rules.extend(factory());
                continue;
            }

            tracing::warn!("unknown plugin `{name}` — skipping");
        }
    }

    // Apply rule configs (severity overrides, disabled rules).
    let applied = apply_rule_config(all_native_rules, &config.rules, &config.overrides);

    // Build the plugin list.
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

    if !applied.rules.is_empty() {
        plugins.push(Box::new(LintRulePlugin::new(applied.rules)));
    }

    // Convert WASM host into plugins.
    #[cfg(feature = "wasm")]
    if let Some(host) = wasm_host {
        plugins.extend(host.into_plugins());
    }

    LoadedPlugins {
        plugins,
        severity_overrides: applied.severity_overrides,
        disabled_rules: applied.disabled_rules,
    }
}

/// Result of applying rule config to a set of native rules.
struct AppliedRuleConfig {
    /// Enabled rules.
    rules: Vec<Box<dyn LintRule>>,
    /// Severity overrides from config.
    severity_overrides: HashMap<String, Severity>,
    /// Rules loaded but disabled by default.
    disabled_rules: HashSet<String>,
}

/// Apply rule severity config and overrides to a set of native rules.
///
/// When `rule_configs` is empty, returns all rules with default severity.
/// Otherwise, only enables rules explicitly listed (or referenced in overrides).
fn apply_rule_config(
    rules: Vec<Box<dyn LintRule>>,
    rule_configs: &HashMap<String, RuleConfig>,
    override_configs: &[starlint_config::Override],
) -> AppliedRuleConfig {
    // Collect rule names referenced in any override block.
    let override_rule_names: HashSet<String> = override_configs
        .iter()
        .flat_map(|ov| ov.rules.keys().cloned())
        .collect();

    if rule_configs.is_empty() {
        return AppliedRuleConfig {
            rules,
            severity_overrides: HashMap::new(),
            disabled_rules: HashSet::new(),
        };
    }

    let mut enabled: Vec<Box<dyn LintRule>> = Vec::new();
    let mut severity_overrides: HashMap<String, Severity> = HashMap::new();
    let mut disabled_rules: HashSet<String> = HashSet::new();

    // Separate configs into exact matches and glob patterns (e.g. "typescript/*").
    let glob_configs: Vec<(&str, &RuleConfig)> = rule_configs
        .iter()
        .filter_map(|(key, config)| key.strip_suffix("/*").map(|prefix| (prefix, config)))
        .collect();

    for mut rule in rules {
        let meta = rule.meta();
        // Exact match takes priority; fall back to glob pattern.
        let in_base = rule_configs.get(&meta.name).or_else(|| {
            meta.name.split_once('/').and_then(|(prefix, _)| {
                glob_configs
                    .iter()
                    .find(|(p, _)| *p == prefix)
                    .map(|(_, config)| *config)
            })
        });
        let in_overrides = override_rule_names.contains(&meta.name);

        match in_base {
            Some(config) => match config {
                RuleConfig::Severity(sev) => match parse_severity(sev) {
                    Ok(Some(severity)) => {
                        if severity != meta.default_severity {
                            severity_overrides.insert(meta.name, severity);
                        }
                        enabled.push(rule);
                    }
                    Ok(None) => {
                        // "off" in base config — load only if referenced in overrides
                        if in_overrides {
                            disabled_rules.insert(meta.name);
                            enabled.push(rule);
                        }
                    }
                    Err(err) => {
                        tracing::warn!("rule `{}`: {err}", meta.name);
                    }
                },
                RuleConfig::Detailed(detailed) => {
                    match parse_severity(&detailed.severity) {
                        Ok(Some(severity)) => {
                            if severity != meta.default_severity {
                                severity_overrides.insert(meta.name.clone(), severity);
                            }
                            let options_value = serde_json::Value::Object(
                                detailed
                                    .options
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect(),
                            );
                            if let Err(err) = rule.configure(&options_value) {
                                tracing::warn!("failed to configure rule `{}`: {err}", meta.name);
                            }
                            enabled.push(rule);
                        }
                        Ok(None) => {
                            // "off" in base config — load only if referenced in overrides
                            if in_overrides {
                                disabled_rules.insert(meta.name);
                                enabled.push(rule);
                            }
                        }
                        Err(err) => {
                            tracing::warn!("rule `{}`: {err}", meta.name);
                        }
                    }
                }
            },
            None => {
                // Not in base config — load as disabled if referenced in overrides
                if in_overrides {
                    disabled_rules.insert(meta.name);
                    enabled.push(rule);
                }
            }
        }
    }

    AppliedRuleConfig {
        rules: enabled,
        severity_overrides,
        disabled_rules,
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use starlint_config::{DetailedRuleConfig, Override, PluginEntry};

    /// Helper: get all rules from a specific native plugin.
    fn plugin_rules(name: &str) -> Vec<Box<dyn LintRule>> {
        native_plugin_registry()
            .into_iter()
            .find(|np| np.name == name)
            .map(|np| (np.factory)())
            .unwrap_or_default()
    }

    #[test]
    fn test_load_plugins_default_config() {
        let config = Config::default();
        let loaded = load_plugins(&config);
        assert!(
            !loaded.plugins.is_empty(),
            "default config should load native plugins"
        );
        assert!(
            loaded.severity_overrides.is_empty(),
            "default config should have no severity overrides"
        );
        assert!(
            loaded.disabled_rules.is_empty(),
            "default config should have no disabled rules"
        );
    }

    #[test]
    fn test_load_plugins_explicit_core_only() {
        let mut config = Config::default();
        config
            .plugins
            .insert("core".to_owned(), PluginEntry::Toggle(true));
        let loaded = load_plugins(&config);
        assert!(
            !loaded.plugins.is_empty(),
            "core plugin should produce rules"
        );
        // Core plugin should have fewer rules than all plugins combined.
        let all_count: usize = native_plugin_registry()
            .into_iter()
            .map(|np| (np.factory)().len())
            .sum();
        let core_count = plugin_rules("core").len();
        assert!(
            core_count < all_count,
            "core rules should be a subset of all rules"
        );
    }

    #[test]
    fn test_load_plugins_disabled_plugin_skipped() {
        let mut config = Config::default();
        config
            .plugins
            .insert("core".to_owned(), PluginEntry::Toggle(false));
        let loaded = load_plugins(&config);
        assert!(
            loaded.plugins.is_empty(),
            "disabled plugin should produce no rules"
        );
    }

    #[test]
    fn test_load_plugins_unknown_plugin_skipped() {
        let mut config = Config::default();
        config
            .plugins
            .insert("nonexistent-plugin".to_owned(), PluginEntry::Toggle(true));
        let loaded = load_plugins(&config);
        assert!(
            loaded.plugins.is_empty(),
            "unknown plugin should produce no rules"
        );
    }

    #[test]
    fn test_apply_rule_config_empty_config() {
        let rules = plugin_rules("core");
        let initial_count = rules.len();
        let applied = apply_rule_config(rules, &HashMap::new(), &[]);
        assert_eq!(
            applied.rules.len(),
            initial_count,
            "empty config should return all rules"
        );
        assert!(
            applied.severity_overrides.is_empty(),
            "empty config should have no severity overrides"
        );
        assert!(
            applied.disabled_rules.is_empty(),
            "empty config should have no disabled rules"
        );
    }

    #[test]
    fn test_apply_rule_config_severity_override() {
        let rules = plugin_rules("core");
        // Find a rule that defaults to "warn" and set it to "error".
        let mut rule_configs = HashMap::new();
        rule_configs.insert(
            "no-debugger".to_owned(),
            RuleConfig::Severity("warn".to_owned()),
        );
        let applied = apply_rule_config(rules, &rule_configs, &[]);
        // Only no-debugger should be enabled (explicit list mode).
        assert_eq!(
            applied.rules.len(),
            1,
            "should only enable explicitly listed rules"
        );
        // no-debugger defaults to Error, setting to warn should produce an override.
        assert!(
            applied.severity_overrides.contains_key("no-debugger"),
            "should have severity override for no-debugger"
        );
        assert_eq!(
            applied.severity_overrides.get("no-debugger").copied(),
            Some(Severity::Warning),
            "severity override should be Warning"
        );
    }

    #[test]
    fn test_apply_rule_config_off_disables() {
        let rules = plugin_rules("core");
        let mut rule_configs = HashMap::new();
        rule_configs.insert(
            "no-debugger".to_owned(),
            RuleConfig::Severity("off".to_owned()),
        );
        let applied = apply_rule_config(rules, &rule_configs, &[]);
        assert!(
            applied.rules.is_empty(),
            "rule set to 'off' with no overrides should be excluded"
        );
    }

    #[test]
    fn test_apply_rule_config_off_with_override_keeps_disabled() {
        let rules = plugin_rules("core");
        let mut rule_configs = HashMap::new();
        rule_configs.insert(
            "no-debugger".to_owned(),
            RuleConfig::Severity("off".to_owned()),
        );
        let overrides = vec![Override {
            files: vec!["**/*.test.js".to_owned()],
            rules: {
                let mut m = HashMap::new();
                m.insert(
                    "no-debugger".to_owned(),
                    RuleConfig::Severity("error".to_owned()),
                );
                m
            },
        }];
        let applied = apply_rule_config(rules, &rule_configs, &overrides);
        assert_eq!(
            applied.rules.len(),
            1,
            "rule turned off but in override should still be loaded"
        );
        assert!(
            applied.disabled_rules.contains("no-debugger"),
            "rule should be in disabled set"
        );
    }

    #[test]
    fn test_apply_rule_config_glob_pattern() {
        let rules = plugin_rules("modules");
        let mut rule_configs = HashMap::new();
        rule_configs.insert("node/*".to_owned(), RuleConfig::Severity("warn".to_owned()));
        let applied = apply_rule_config(rules, &rule_configs, &[]);
        assert!(
            !applied.rules.is_empty(),
            "glob pattern should match node/ prefixed rules"
        );
        for meta in applied.rules.iter().map(|r| r.meta()) {
            assert!(
                meta.name.starts_with("node/"),
                "only node/ prefixed rules should be enabled, got: {}",
                meta.name
            );
        }
    }

    #[test]
    fn test_apply_rule_config_detailed_with_options() {
        let rules = plugin_rules("core");
        let mut rule_configs = HashMap::new();
        rule_configs.insert(
            "no-console".to_owned(),
            RuleConfig::Detailed(DetailedRuleConfig {
                severity: "error".to_owned(),
                options: HashMap::new(),
            }),
        );
        let applied = apply_rule_config(rules, &rule_configs, &[]);
        assert_eq!(
            applied.rules.len(),
            1,
            "should enable one rule with detailed config"
        );
    }

    #[test]
    fn test_apply_rule_config_invalid_severity() {
        let rules = plugin_rules("core");
        let mut rule_configs = HashMap::new();
        rule_configs.insert(
            "no-debugger".to_owned(),
            RuleConfig::Severity("invalid-severity".to_owned()),
        );
        let applied = apply_rule_config(rules, &rule_configs, &[]);
        assert!(
            applied.rules.is_empty(),
            "invalid severity should not enable any rules"
        );
    }

    #[test]
    fn test_apply_rule_config_not_in_base_not_in_override() {
        let rules = plugin_rules("core");
        // Config only mentions a rule that doesn't exist in the rule set.
        let mut rule_configs = HashMap::new();
        rule_configs.insert(
            "nonexistent-rule".to_owned(),
            RuleConfig::Severity("error".to_owned()),
        );
        let applied = apply_rule_config(rules, &rule_configs, &[]);
        assert!(
            applied.rules.is_empty(),
            "rules not in config should be excluded"
        );
    }

    #[test]
    fn test_load_plugins_with_severity_override() {
        let mut config = Config::default();
        config
            .plugins
            .insert("core".to_owned(), PluginEntry::Toggle(true));
        config.rules.insert(
            "no-debugger".to_owned(),
            RuleConfig::Severity("warn".to_owned()),
        );
        let loaded = load_plugins(&config);
        assert!(
            loaded.severity_overrides.contains_key("no-debugger"),
            "should have severity override from config"
        );
    }

    #[test]
    fn test_load_plugins_multiple_native_plugins() {
        let mut config = Config::default();
        config
            .plugins
            .insert("core".to_owned(), PluginEntry::Toggle(true));
        config
            .plugins
            .insert("react".to_owned(), PluginEntry::Toggle(true));
        let loaded = load_plugins(&config);
        assert!(!loaded.plugins.is_empty(), "multiple plugins should load");
        // Should have rules from both plugins.
        let total_rules: usize = loaded.plugins.iter().map(|p| p.rules().len()).sum();
        let core_count = plugin_rules("core").len();
        assert!(
            total_rules > core_count,
            "multiple plugins should have more rules than core alone"
        );
    }
}
