//! Built-in native lint rules and rule registry.
//!
//! All rules are registered in [`all_rules`]. The [`rules_for_config`] function
//! filters and configures rules based on a rule config map.

pub mod array_callback_return;
pub mod catch_error_name;
pub mod constructor_super;
pub mod eqeqeq;
pub mod for_direction;
pub mod getter_return;
pub mod max_complexity;
pub mod new_for_builtins;
pub mod no_async_promise_executor;
pub mod no_compare_neg_zero;
pub mod no_console;
pub mod no_console_spaces;
pub mod no_constant_binary_expression;
pub mod no_constant_condition;
pub mod no_case_declarations;
pub mod no_cond_assign;
pub mod no_constructor_return;
pub mod no_control_regex;
pub mod no_debugger;
pub mod no_delete_var;
pub mod no_dupe_class_members;
pub mod no_dupe_else_if;
pub mod no_dupe_keys;
pub mod no_duplicate_case;
pub mod no_empty;
pub mod no_empty_character_class;
pub mod no_empty_pattern;
pub mod no_empty_static_block;
pub mod no_ex_assign;
pub mod no_extra_semi;
pub mod no_fallthrough;
pub mod no_inner_declarations;
pub mod no_irregular_whitespace;
pub mod no_lonely_if;
pub mod no_loss_of_precision;
pub mod no_nested_ternary;
pub mod no_new_native_nonconstructor;
pub mod no_nonoctal_decimal_escape;
pub mod no_obj_calls;
pub mod no_promise_executor_return;
pub mod no_prototype_builtins;
pub mod no_regex_spaces;
pub mod no_return_assign;
pub mod no_script_url;
pub mod no_self_assign;
pub mod no_sequences;
pub mod no_self_compare;
pub mod no_setter_return;
pub mod no_shadow_restricted_names;
pub mod no_ternary;
pub mod no_sparse_arrays;
pub mod no_template_curly_in_string;
pub mod no_this_before_super;
pub mod no_throw_literal;
pub mod no_typeof_undefined;
pub mod no_undefined;
pub mod no_unneeded_ternary;
pub mod no_unsafe_finally;
pub mod no_unexpected_multiline;
pub mod no_unsafe_negation;
pub mod no_unreachable;
pub mod no_unsafe_optional_chaining;
pub mod no_unused_labels;
pub mod no_unused_private_class_members;
pub mod no_useless_call;
pub mod no_useless_catch;
pub mod no_useless_computed_key;
pub mod no_useless_concat;
pub mod no_useless_constructor;
pub mod no_useless_rename;
pub mod no_with;
pub mod no_var;
pub mod no_zero_fractions;
pub mod number_literal_case;
pub mod operator_assignment;
pub mod prefer_includes;
pub mod prefer_object_has_own;
pub mod prefer_optional_catch_binding;
pub mod prefer_rest_params;
pub mod prefer_spread;
pub mod prefer_template;
pub mod radix;
pub mod symbol_description;
pub mod throw_new_error;
pub mod use_isnan;
pub mod valid_typeof;
pub mod yoda;

use std::collections::{HashMap, HashSet};

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
        Box::new(no_lonely_if::NoLonelyIf),
        Box::new(no_unneeded_ternary::NoUnneededTernary),
        Box::new(no_useless_catch::NoUselessCatch),
        Box::new(no_useless_rename::NoUselessRename),
        Box::new(eqeqeq::Eqeqeq),
        Box::new(no_var::NoVar),
        Box::new(max_complexity::MaxComplexity::new()),
        Box::new(no_zero_fractions::NoZeroFractions),
        Box::new(number_literal_case::NumberLiteralCase),
        Box::new(no_nested_ternary::NoNestedTernary),
        Box::new(throw_new_error::ThrowNewError),
        Box::new(no_typeof_undefined::NoTypeofUndefined),
        Box::new(catch_error_name::CatchErrorName::new()),
        Box::new(new_for_builtins::NewForBuiltins),
        Box::new(no_console_spaces::NoConsoleSpaces),
        Box::new(prefer_optional_catch_binding::PreferOptionalCatchBinding),
        Box::new(prefer_includes::PreferIncludes),
        Box::new(for_direction::ForDirection),
        Box::new(no_compare_neg_zero::NoCompareNegZero),
        Box::new(no_dupe_keys::NoDupeKeys),
        Box::new(no_duplicate_case::NoDuplicateCase),
        Box::new(no_sparse_arrays::NoSparseArrays),
        Box::new(valid_typeof::ValidTypeof),
        Box::new(use_isnan::UseIsnan),
        Box::new(no_self_assign::NoSelfAssign),
        Box::new(no_self_compare::NoSelfCompare),
        Box::new(no_empty_pattern::NoEmptyPattern),
        Box::new(no_delete_var::NoDeleteVar),
        Box::new(no_empty_static_block::NoEmptyStaticBlock),
        Box::new(no_obj_calls::NoObjCalls),
        Box::new(no_template_curly_in_string::NoTemplateCurlyInString),
        Box::new(no_async_promise_executor::NoAsyncPromiseExecutor),
        Box::new(no_shadow_restricted_names::NoShadowRestrictedNames),
        Box::new(no_case_declarations::NoCaseDeclarations),
        Box::new(no_ex_assign::NoExAssign),
        Box::new(no_dupe_class_members::NoDupeClassMembers),
        Box::new(no_new_native_nonconstructor::NoNewNativeNonconstructor),
        Box::new(no_unsafe_negation::NoUnsafeNegation),
        Box::new(no_unused_labels::NoUnusedLabels),
        Box::new(no_prototype_builtins::NoPrototypeBuiltins),
        Box::new(no_nonoctal_decimal_escape::NoNonoctalDecimalEscape),
        Box::new(no_loss_of_precision::NoLossOfPrecision),
        Box::new(no_setter_return::NoSetterReturn),
        Box::new(getter_return::GetterReturn),
        Box::new(no_cond_assign::NoCondAssign),
        Box::new(no_unsafe_finally::NoUnsafeFinally),
        Box::new(no_constructor_return::NoConstructorReturn),
        Box::new(constructor_super::ConstructorSuper),
        Box::new(no_irregular_whitespace::NoIrregularWhitespace),
        Box::new(no_promise_executor_return::NoPromiseExecutorReturn),
        Box::new(no_dupe_else_if::NoDupeElseIf),
        Box::new(no_unsafe_optional_chaining::NoUnsafeOptionalChaining),
        Box::new(no_inner_declarations::NoInnerDeclarations),
        Box::new(no_unreachable::NoUnreachable),
        Box::new(no_this_before_super::NoThisBeforeSuper),
        Box::new(no_constant_binary_expression::NoConstantBinaryExpression),
        Box::new(array_callback_return::ArrayCallbackReturn),
        Box::new(no_unexpected_multiline::NoUnexpectedMultiline),
        Box::new(no_regex_spaces::NoRegexSpaces),
        Box::new(no_empty_character_class::NoEmptyCharacterClass),
        Box::new(no_control_regex::NoControlRegex),
        Box::new(no_fallthrough::NoFallthrough),
        Box::new(no_unused_private_class_members::NoUnusedPrivateClassMembers),
        Box::new(no_useless_constructor::NoUselessConstructor),
        Box::new(no_throw_literal::NoThrowLiteral),
        Box::new(no_script_url::NoScriptUrl),
        Box::new(no_return_assign::NoReturnAssign),
        Box::new(no_sequences::NoSequences),
        Box::new(no_useless_computed_key::NoUselessComputedKey),
        Box::new(symbol_description::SymbolDescription),
        Box::new(prefer_object_has_own::PreferObjectHasOwn),
        Box::new(no_with::NoWith),
        Box::new(no_useless_concat::NoUselessConcat),
        Box::new(no_useless_call::NoUselessCall),
        Box::new(no_undefined::NoUndefined),
        Box::new(no_ternary::NoTernary),
        Box::new(prefer_template::PreferTemplate),
        Box::new(yoda::Yoda),
        Box::new(radix::Radix),
        Box::new(prefer_spread::PreferSpread),
        Box::new(prefer_rest_params::PreferRestParams),
        Box::new(operator_assignment::OperatorAssignment),
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
    /// Rules loaded but disabled by default (only active via file-pattern overrides).
    pub disabled_rules: HashSet<String>,
}

/// Build a rule set from config, including rules referenced in overrides.
///
/// If `rule_configs` is empty, returns all rules with their default severity.
/// Otherwise, **only** enables rules that appear in `rule_configs` or
/// `override_configs` (unless "off" in base and not in overrides).
///
/// Rules referenced only in overrides are loaded but added to
/// [`ConfiguredRules::disabled_rules`] — their diagnostics are suppressed
/// by default and only activated for files matching an override pattern.
///
/// Configured severities are returned in [`ConfiguredRules::severity_overrides`]
/// so the engine can apply them to diagnostics.
#[must_use]
pub fn rules_for_config<S: ::std::hash::BuildHasher>(
    rule_configs: &HashMap<String, starlint_config::RuleConfig, S>,
    override_configs: &[starlint_config::Override],
) -> ConfiguredRules {
    // Collect rule names referenced in any override block.
    let override_rule_names: HashSet<String> = override_configs
        .iter()
        .flat_map(|ov| ov.rules.keys().cloned())
        .collect();

    if rule_configs.is_empty() {
        return ConfiguredRules {
            rules: all_rules(),
            severity_overrides: HashMap::new(),
            disabled_rules: HashSet::new(),
        };
    }

    let available = all_rules();
    let mut enabled: Vec<Box<dyn NativeRule>> = Vec::new();
    let mut severity_overrides: HashMap<String, Severity> = HashMap::new();
    let mut disabled_rules: HashSet<String> = HashSet::new();

    for mut rule in available {
        let meta = rule.meta();
        let in_base = rule_configs.get(&meta.name);
        let in_overrides = override_rule_names.contains(&meta.name);

        match in_base {
            Some(config) => match config {
                starlint_config::RuleConfig::Severity(sev) => match parse_severity(sev) {
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

    ConfiguredRules {
        rules: enabled,
        severity_overrides,
        disabled_rules,
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
        let configured = rules_for_config(&configs, &[]);
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
        let configured = rules_for_config(&configs, &[]);
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
        let configured = rules_for_config(&configs, &[]);
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
        let configured = rules_for_config(&configs, &[]);
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
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.severity_overrides.is_empty(),
            "no override when severity matches default"
        );
    }

    #[test]
    fn test_empty_overrides_no_disabled_rules() {
        let configs = HashMap::new();
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.disabled_rules.is_empty(),
            "empty overrides should produce no disabled rules"
        );
    }

    #[test]
    fn test_override_only_rule_loaded_as_disabled() {
        // Base config enables only no-debugger; override references no-console
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        let overrides = vec![starlint_config::Override {
            files: vec!["**/*.test.ts".to_owned()],
            rules: std::iter::once((
                "no-console".to_owned(),
                starlint_config::RuleConfig::Severity("warn".to_owned()),
            ))
            .collect(),
        }];
        let configured = rules_for_config(&configs, &overrides);

        let names: Vec<String> = configured.rules.iter().map(|r| r.meta().name).collect();
        assert!(
            names.contains(&"no-debugger".to_owned()),
            "base rule should be loaded"
        );
        assert!(
            names.contains(&"no-console".to_owned()),
            "override-only rule should be loaded"
        );
        assert!(
            configured.disabled_rules.contains("no-console"),
            "override-only rule should be in disabled_rules"
        );
        assert!(
            !configured.disabled_rules.contains("no-debugger"),
            "base rule should not be disabled"
        );
    }

    #[test]
    fn test_off_rule_loaded_when_in_override() {
        // Base: no-debugger = "off", override references no-debugger
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("off".to_owned()),
        );
        let overrides = vec![starlint_config::Override {
            files: vec!["**/*.test.ts".to_owned()],
            rules: std::iter::once((
                "no-debugger".to_owned(),
                starlint_config::RuleConfig::Severity("error".to_owned()),
            ))
            .collect(),
        }];
        let configured = rules_for_config(&configs, &overrides);

        assert_eq!(
            configured.rules.len(),
            1,
            "off rule referenced in override should be loaded"
        );
        assert!(
            configured.disabled_rules.contains("no-debugger"),
            "off rule should be in disabled_rules"
        );
    }

    #[test]
    fn test_off_rule_skipped_when_not_in_override() {
        // Base: no-debugger = "off", no overrides reference it
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("off".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.rules.is_empty(),
            "off rule with no override should be skipped"
        );
    }
}
