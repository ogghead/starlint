//! Jsdoc lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! jsdoc rules.

pub mod rules;

starlint_rule_framework::declare_plugin! {
    name: "jsdoc",
    rules: [
        crate::rules::jsdoc::check_access::CheckAccess,
        crate::rules::jsdoc::check_param_names::CheckParamNames,
        crate::rules::jsdoc::check_property_names::CheckPropertyNames,
        crate::rules::jsdoc::check_tag_names::CheckTagNames,
        crate::rules::jsdoc::check_types::CheckTypes,
        crate::rules::jsdoc::check_values::CheckValues,
        crate::rules::jsdoc::empty_tags::EmptyTags,
        crate::rules::jsdoc::implements_on_classes::ImplementsOnClasses,
        crate::rules::jsdoc::match_description::MatchDescription,
        crate::rules::jsdoc::match_name::MatchName,
        crate::rules::jsdoc::no_defaults::NoDefaults,
        crate::rules::jsdoc::no_multi_asterisks::NoMultiAsterisks,
        crate::rules::jsdoc::no_restricted_syntax::NoRestrictedSyntax,
        crate::rules::jsdoc::require_description::RequireDescription,
        crate::rules::jsdoc::require_param::RequireParam,
        crate::rules::jsdoc::require_param_description::RequireParamDescription,
        crate::rules::jsdoc::require_param_type::RequireParamType,
        crate::rules::jsdoc::require_returns::RequireReturns,
    ]
}
