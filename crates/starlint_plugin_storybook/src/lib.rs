//! Storybook lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! storybook rules.

pub mod rules;

starlint_rule_framework::declare_plugin! {
    name: "storybook",
    rules: [
        crate::rules::storybook::await_interactions::AwaitInteractions,
        crate::rules::storybook::context_in_play_function::ContextInPlayFunction,
        crate::rules::storybook::csf_component::CsfComponent,
        crate::rules::storybook::default_exports::DefaultExports,
        crate::rules::storybook::hierarchy_separator::HierarchySeparator,
        crate::rules::storybook::meta_inline_properties::MetaInlineProperties,
        crate::rules::storybook::meta_satisfies_type::MetaSatisfiesType,
        crate::rules::storybook::no_redundant_story_name::NoRedundantStoryName,
        crate::rules::storybook::no_stories_of::NoStoriesOf,
        crate::rules::storybook::no_title_property_in_meta::NoTitlePropertyInMeta,
        crate::rules::storybook::no_uninstalled_addons::NoUninstalledAddons,
        crate::rules::storybook::prefer_pascal_case::PreferPascalCase,
        crate::rules::storybook::story_exports::StoryExports,
        crate::rules::storybook::use_storybook_expect::UseStorybookExpect,
        crate::rules::storybook::use_storybook_testing_library::UseStorybookTestingLibrary,
    ]
}
