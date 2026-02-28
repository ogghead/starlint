//! Storybook-specific lint rules.
//!
//! Rules are prefixed with `storybook/` in config and output.

pub mod await_interactions;
pub mod context_in_play_function;
pub mod csf_component;
pub mod default_exports;
pub mod hierarchy_separator;
pub mod meta_inline_properties;
pub mod meta_satisfies_type;
pub mod no_redundant_story_name;
pub mod no_stories_of;
pub mod no_title_property_in_meta;
pub mod no_uninstalled_addons;
pub mod prefer_pascal_case;
pub mod story_exports;
pub mod use_storybook_expect;
pub mod use_storybook_testing_library;

use crate::rule::NativeRule;

/// Return all Storybook rules.
#[must_use]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(await_interactions::AwaitInteractions),
        Box::new(context_in_play_function::ContextInPlayFunction),
        Box::new(csf_component::CsfComponent),
        Box::new(default_exports::DefaultExports),
        Box::new(hierarchy_separator::HierarchySeparator),
        Box::new(meta_inline_properties::MetaInlineProperties),
        Box::new(meta_satisfies_type::MetaSatisfiesType),
        Box::new(no_redundant_story_name::NoRedundantStoryName),
        Box::new(no_stories_of::NoStoriesOf),
        Box::new(no_title_property_in_meta::NoTitlePropertyInMeta),
        Box::new(no_uninstalled_addons::NoUninstalledAddons),
        Box::new(prefer_pascal_case::PreferPascalCase),
        Box::new(story_exports::StoryExports),
        Box::new(use_storybook_expect::UseStorybookExpect),
        Box::new(use_storybook_testing_library::UseStorybookTestingLibrary),
    ]
}
