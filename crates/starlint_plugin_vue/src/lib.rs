//! Vue lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! vue rules.

pub mod rules;

use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

/// Create the vue plugin with all its rules.
#[must_use]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LintRulePlugin::new(all_rules()))
}

/// Return all vue lint rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(
            crate::rules::vue::component_definition_name_casing::ComponentDefinitionNameCasing,
        ),
        Box::new(crate::rules::vue::custom_event_name_casing::CustomEventNameCasing),
        Box::new(crate::rules::vue::html_closing_bracket_newline::HtmlClosingBracketNewline),
        Box::new(crate::rules::vue::html_self_closing::HtmlSelfClosing),
        Box::new(crate::rules::vue::no_arrow_functions_in_watch::NoArrowFunctionsInWatch),
        Box::new(crate::rules::vue::no_async_in_computed_properties::NoAsyncInComputedProperties),
        Box::new(crate::rules::vue::no_child_content::NoChildContent),
        Box::new(crate::rules::vue::no_component_options_typo::NoComponentOptionsTypo),
        Box::new(crate::rules::vue::no_dupe_keys::NoDupeKeys),
        Box::new(crate::rules::vue::no_expose_after_await::NoExposeAfterAwait),
        Box::new(crate::rules::vue::no_lifecycle_after_await::NoLifecycleAfterAwait),
        Box::new(crate::rules::vue::no_ref_object_reactivity_loss::NoRefObjectReactivityLoss),
        Box::new(crate::rules::vue::no_reserved_component_names::NoReservedComponentNames),
        Box::new(crate::rules::vue::no_setup_props_reactivity_loss::NoSetupPropsReactivityLoss),
        Box::new(crate::rules::vue::no_watch_after_await::NoWatchAfterAwait),
        Box::new(crate::rules::vue::prefer_define_options::PreferDefineOptions),
        Box::new(crate::rules::vue::require_prop_comment::RequirePropComment),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_plugin_returns_rules() {
        let plugin = create_plugin();
        let rules = plugin.rules();
        assert!(
            !rules.is_empty(),
            "vue plugin should provide at least one rule"
        );
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 17, "vue should have 17 rules");
    }
}
