//! Unified plugin loader for starlint.
//!
//! Resolves plugin names from config into [`Plugin`] trait objects,
//! handling both native Rust rule bundles and WASM plugins through
//! a single code path.

use std::collections::{HashMap, HashSet};

use starlint_config::{Config, RuleConfig};
use starlint_core::lint_rule::LintRule;
use starlint_core::lint_rule_plugin::LintRulePlugin;
use starlint_core::plugin::Plugin;
use starlint_core::rules::{NativePlugin, native_plugin_registry, parse_severity};
use starlint_plugin_sdk::diagnostic::Severity;

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

/// Factory function type for native plugin rule bundles.
type RuleFactory = fn() -> Vec<Box<dyn LintRule>>;

/// Load all plugins from config.
///
/// When the `[plugins]` section is absent or empty, all built-in native
/// plugins are enabled by default. When plugins are listed explicitly,
/// only those plugins are loaded.
///
/// Resolution order for each plugin name:
/// 1. If `path` specified → external WASM from disk
/// 2. If name matches native registry → native `LintRulePlugin`
/// 3. If name matches embedded WASM → built-in WASM
/// 4. Otherwise → warning logged, plugin skipped
#[allow(clippy::module_name_repetitions)]
pub fn load_plugins(config: &Config) -> LoadedPlugins {
    let registry: HashMap<&str, RuleFactory> = native_plugin_registry()
        .into_iter()
        .map(|NativePlugin { name, factory }| (name, factory))
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

            // Check native registry first.
            if let Some(factory) = registry.get(name.as_str()) {
                all_native_rules.extend(factory());
                continue;
            }

            // Check embedded WASM builtins.
            #[cfg(feature = "wasm")]
            if let Some(wasm_name) = starlint_wasm_host::builtins::config_to_wasm_name(name) {
                if starlint_wasm_host::builtins::get_builtin_bytes(wasm_name).is_some() {
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
                    let mut active = HashSet::new();
                    active.insert(name.clone());
                    if let Err(err) = host.load_builtins(&active) {
                        tracing::warn!("failed to load built-in WASM plugin `{name}`: {err}");
                    }
                    continue;
                }
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
