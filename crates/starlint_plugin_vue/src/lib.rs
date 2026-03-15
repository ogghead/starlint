//! Vue lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! vue rules.

pub mod rules;

starlint_rule_framework::declare_plugin! {
    name: "vue",
    rules: [
        crate::rules::vue::component_definition_name_casing::ComponentDefinitionNameCasing,
        crate::rules::vue::custom_event_name_casing::CustomEventNameCasing,
        crate::rules::vue::html_closing_bracket_newline::HtmlClosingBracketNewline,
        crate::rules::vue::html_self_closing::HtmlSelfClosing,
        crate::rules::vue::no_arrow_functions_in_watch::NoArrowFunctionsInWatch,
        crate::rules::vue::no_async_in_computed_properties::NoAsyncInComputedProperties,
        crate::rules::vue::no_child_content::NoChildContent,
        crate::rules::vue::no_component_options_typo::NoComponentOptionsTypo,
        crate::rules::vue::no_dupe_keys::NoDupeKeys,
        crate::rules::vue::no_expose_after_await::NoExposeAfterAwait,
        crate::rules::vue::no_lifecycle_after_await::NoLifecycleAfterAwait,
        crate::rules::vue::no_ref_object_reactivity_loss::NoRefObjectReactivityLoss,
        crate::rules::vue::no_reserved_component_names::NoReservedComponentNames,
        crate::rules::vue::no_setup_props_reactivity_loss::NoSetupPropsReactivityLoss,
        crate::rules::vue::no_watch_after_await::NoWatchAfterAwait,
        crate::rules::vue::prefer_define_options::PreferDefineOptions,
        crate::rules::vue::require_prop_comment::RequirePropComment,
    ]
}
