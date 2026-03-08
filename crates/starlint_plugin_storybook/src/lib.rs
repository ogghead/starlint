//! Storybook lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! storybook rules.

pub mod rules;

use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

/// Create the storybook plugin with all its rules.
#[must_use]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LintRulePlugin::new(all_rules()))
}

/// Return all storybook lint rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(crate::rules::storybook::await_interactions::AwaitInteractions),
        Box::new(crate::rules::storybook::context_in_play_function::ContextInPlayFunction),
        Box::new(crate::rules::storybook::csf_component::CsfComponent),
        Box::new(crate::rules::storybook::default_exports::DefaultExports),
        Box::new(crate::rules::storybook::hierarchy_separator::HierarchySeparator),
        Box::new(crate::rules::storybook::meta_inline_properties::MetaInlineProperties),
        Box::new(crate::rules::storybook::meta_satisfies_type::MetaSatisfiesType),
        Box::new(crate::rules::storybook::no_redundant_story_name::NoRedundantStoryName),
        Box::new(crate::rules::storybook::no_stories_of::NoStoriesOf),
        Box::new(crate::rules::storybook::no_title_property_in_meta::NoTitlePropertyInMeta),
        Box::new(crate::rules::storybook::no_uninstalled_addons::NoUninstalledAddons),
        Box::new(crate::rules::storybook::prefer_pascal_case::PreferPascalCase),
        Box::new(crate::rules::storybook::story_exports::StoryExports),
        Box::new(crate::rules::storybook::use_storybook_expect::UseStorybookExpect),
        Box::new(
            crate::rules::storybook::use_storybook_testing_library::UseStorybookTestingLibrary,
        ),
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
            "storybook plugin should provide at least one rule"
        );
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 15, "storybook should have 15 rules");
    }
}
