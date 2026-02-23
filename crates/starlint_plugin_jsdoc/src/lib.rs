//! Jsdoc lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! jsdoc rules.

pub mod rules;

use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

/// Create the jsdoc plugin with all its rules.
#[must_use]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LintRulePlugin::new(all_rules()))
}

/// Return all jsdoc lint rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(crate::rules::jsdoc::check_access::CheckAccess),
        Box::new(crate::rules::jsdoc::check_param_names::CheckParamNames),
        Box::new(crate::rules::jsdoc::check_property_names::CheckPropertyNames),
        Box::new(crate::rules::jsdoc::check_tag_names::CheckTagNames),
        Box::new(crate::rules::jsdoc::check_types::CheckTypes),
        Box::new(crate::rules::jsdoc::check_values::CheckValues),
        Box::new(crate::rules::jsdoc::empty_tags::EmptyTags),
        Box::new(crate::rules::jsdoc::implements_on_classes::ImplementsOnClasses),
        Box::new(crate::rules::jsdoc::match_description::MatchDescription),
        Box::new(crate::rules::jsdoc::match_name::MatchName),
        Box::new(crate::rules::jsdoc::no_defaults::NoDefaults),
        Box::new(crate::rules::jsdoc::no_multi_asterisks::NoMultiAsterisks),
        Box::new(crate::rules::jsdoc::no_restricted_syntax::NoRestrictedSyntax),
        Box::new(crate::rules::jsdoc::require_description::RequireDescription),
        Box::new(crate::rules::jsdoc::require_param::RequireParam),
        Box::new(crate::rules::jsdoc::require_param_description::RequireParamDescription),
        Box::new(crate::rules::jsdoc::require_param_type::RequireParamType),
        Box::new(crate::rules::jsdoc::require_returns::RequireReturns),
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
            "jsdoc plugin should provide at least one rule"
        );
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 18, "jsdoc should have 18 rules");
    }
}
