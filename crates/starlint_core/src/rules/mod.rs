//! Built-in native lint rules and rule registry.
//!
//! All rules are registered in [`all_rules`]. The [`rules_for_config`] function
//! filters and configures rules based on a rule config map.

pub mod eqeqeq;
pub mod no_console;
pub mod no_constant_condition;
pub mod no_debugger;
pub mod no_empty;
pub mod no_extra_semi;
pub mod no_var;

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::RuleMeta;

use crate::rule::NativeRule;

/// Return all built-in native rules with their default configuration.
#[must_use]
pub fn all_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(no_debugger::NoDebugger),
        Box::new(no_console::NoConsole),
        Box::new(no_constant_condition::NoConstantCondition),
        Box::new(no_empty::NoEmpty),
        Box::new(no_extra_semi::NoExtraSemi),
        Box::new(eqeqeq::Eqeqeq),
        Box::new(no_var::NoVar),
    ]
}

/// Return metadata for all built-in native rules.
#[must_use]
pub fn all_rule_metas() -> Vec<RuleMeta> {
    all_rules().iter().map(|r| r.meta()).collect()
}

/// Parse a severity string from config into a [`Severity`].
///
/// Returns `Ok(None)` for "off" (rule disabled).
/// Returns `Err` for unrecognized severity strings.
pub fn parse_severity(s: &str) -> Result<Option<Severity>, String> {
    match s {
        "error" => Ok(Some(Severity::Error)),
        "warn" | "warning" => Ok(Some(Severity::Warning)),
        "off" => Ok(None),
        _ => Err(format!(
            "unknown severity `{s}`; expected \"error\", \"warn\", or \"off\""
        )),
    }
}

/// Rules and their configured severity overrides.
pub struct ConfiguredRules {
    /// Enabled native rules.
    pub rules: Vec<Box<dyn NativeRule>>,
    /// Severity overrides from config (rule name → configured severity).
    pub severity_overrides: HashMap<String, Severity>,
}

/// Build a rule set from config.
///
/// If `rule_configs` is empty, returns all rules with their default severity.
/// Otherwise, **only** enables rules that appear in the config (unless "off").
/// Rules not mentioned in config are silently disabled. For example, setting
/// `"no-debugger" = "error"` in `[rules]` will disable all other built-in rules.
/// To override a single rule while keeping all defaults, add all desired rules
/// to the `[rules]` section.
///
/// Configured severities are returned in [`ConfiguredRules::severity_overrides`]
/// so the engine can apply them to diagnostics.
#[must_use]
pub fn rules_for_config<S: ::std::hash::BuildHasher>(
    rule_configs: &HashMap<String, starlint_config::RuleConfig, S>,
) -> ConfiguredRules {
    if rule_configs.is_empty() {
        return ConfiguredRules {
            rules: all_rules(),
            severity_overrides: HashMap::new(),
        };
    }

    let available = all_rules();
    let mut enabled: Vec<Box<dyn NativeRule>> = Vec::new();
    let mut severity_overrides: HashMap<String, Severity> = HashMap::new();

    for mut rule in available {
        let meta = rule.meta();
        if let Some(config) = rule_configs.get(&meta.name) {
            match config {
                starlint_config::RuleConfig::Severity(sev) => {
                    match parse_severity(sev) {
                        Ok(Some(severity)) => {
                            if severity != meta.default_severity {
                                severity_overrides.insert(meta.name, severity);
                            }
                            enabled.push(rule);
                        }
                        Ok(None) => {} // "off" — skip rule
                        Err(err) => {
                            tracing::warn!("rule `{}`: {err}", meta.name);
                        }
                    }
                }
                starlint_config::RuleConfig::Detailed(detailed) => {
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
                        Ok(None) => {} // "off" — skip rule
                        Err(err) => {
                            tracing::warn!("rule `{}`: {err}", meta.name);
                        }
                    }
                }
            }
        }
    }

    ConfiguredRules {
        rules: enabled,
        severity_overrides,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_returns_builtin_rules() {
        let rules = all_rules();
        assert!(rules.len() >= 2, "should have at least 2 built-in rules");

        let names: Vec<String> = rules.iter().map(|r| r.meta().name).collect();
        assert!(
            names.contains(&"no-debugger".to_owned()),
            "should contain no-debugger"
        );
        assert!(
            names.contains(&"no-console".to_owned()),
            "should contain no-console"
        );
    }

    #[test]
    fn test_all_rule_metas() {
        let metas = all_rule_metas();
        assert!(metas.len() >= 2, "should have metadata for all rules");
    }

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("error"), Ok(Some(Severity::Error)));
        assert_eq!(parse_severity("warn"), Ok(Some(Severity::Warning)));
        assert_eq!(parse_severity("warning"), Ok(Some(Severity::Warning)));
        assert_eq!(parse_severity("off"), Ok(None));
    }

    #[test]
    fn test_parse_severity_rejects_unknown() {
        assert!(
            parse_severity("errror").is_err(),
            "typo should be rejected, not silently treated as error"
        );
        assert!(
            parse_severity("whatever").is_err(),
            "unknown severity should be rejected"
        );
    }

    #[test]
    fn test_rules_for_empty_config() {
        let configs = HashMap::new();
        let configured = rules_for_config(&configs);
        assert!(
            configured.rules.len() >= 2,
            "empty config should return all default rules"
        );
        assert!(
            configured.severity_overrides.is_empty(),
            "empty config should have no severity overrides"
        );
    }

    #[test]
    fn test_rules_for_config_filters() {
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        let configured = rules_for_config(&configs);
        assert_eq!(
            configured.rules.len(),
            1,
            "should only enable configured rules"
        );
        assert_eq!(
            configured.rules.first().map(|r| r.meta().name),
            Some("no-debugger".to_owned()),
            "should be no-debugger rule"
        );
    }

    #[test]
    fn test_rules_for_config_off() {
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("off".to_owned()),
        );
        let configured = rules_for_config(&configs);
        assert!(
            configured.rules.is_empty(),
            "rule set to 'off' should not be enabled"
        );
    }

    #[test]
    fn test_severity_override_applied() {
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("warn".to_owned()),
        );
        let configured = rules_for_config(&configs);
        assert_eq!(configured.rules.len(), 1, "should enable the rule");
        assert_eq!(
            configured.severity_overrides.get("no-debugger"),
            Some(&Severity::Warning),
            "no-debugger should be overridden to Warning"
        );
    }

    #[test]
    fn test_no_override_when_severity_matches_default() {
        let mut configs = HashMap::new();
        // no-debugger default is Error, so setting "error" should not create an override
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        let configured = rules_for_config(&configs);
        assert!(
            configured.severity_overrides.is_empty(),
            "no override when severity matches default"
        );
    }
}
