//! React lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! react rules.

pub mod rules;

use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

/// Create the react plugin with all its rules.
#[must_use]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LintRulePlugin::new(all_rules()))
}

/// Return all react lint rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(crate::rules::react::button_has_type::ButtonHasType),
        Box::new(crate::rules::react::checked_requires_onchange_or_readonly::CheckedRequiresOnchangeOrReadonly),
        Box::new(crate::rules::react::display_name::DisplayName),
        Box::new(crate::rules::react::exhaustive_deps::ExhaustiveDeps),
        Box::new(crate::rules::react::forbid_dom_props::ForbidDomProps),
        Box::new(crate::rules::react::forbid_elements::ForbidElements),
        Box::new(crate::rules::react::forward_ref_uses_ref::ForwardRefUsesRef),
        Box::new(crate::rules::react::iframe_missing_sandbox::IframeMissingSandbox),
        Box::new(crate::rules::react::jsx_boolean_value::JsxBooleanValue),
        Box::new(crate::rules::react::jsx_curly_brace_presence::JsxCurlyBracePresence),
        Box::new(crate::rules::react::jsx_filename_extension::JsxFilenameExtension),
        Box::new(crate::rules::react::jsx_fragments::JsxFragments),
        Box::new(crate::rules::react::jsx_handler_names::JsxHandlerNames),
        Box::new(crate::rules::react::jsx_key::JsxKey),
        Box::new(crate::rules::react::jsx_max_depth::JsxMaxDepth),
        Box::new(crate::rules::react::jsx_no_comment_textnodes::JsxNoCommentTextnodes),
        Box::new(
            crate::rules::react::jsx_no_constructed_context_values::JsxNoConstructedContextValues,
        ),
        Box::new(crate::rules::react::jsx_no_duplicate_props::JsxNoDuplicateProps),
        Box::new(crate::rules::react::jsx_no_script_url::JsxNoScriptUrl),
        Box::new(crate::rules::react::jsx_no_target_blank::JsxNoTargetBlank),
        Box::new(crate::rules::react::jsx_no_undef::JsxNoUndef),
        Box::new(crate::rules::react::jsx_no_useless_fragment::JsxNoUselessFragment),
        Box::new(crate::rules::react::jsx_pascal_case::JsxPascalCase),
        Box::new(crate::rules::react::jsx_props_no_spread_multi::JsxPropsNoSpreadMulti),
        Box::new(crate::rules::react::jsx_props_no_spreading::JsxPropsNoSpreading),
        Box::new(crate::rules::react::no_array_index_key::NoArrayIndexKey),
        Box::new(crate::rules::react::no_children_prop::NoChildrenProp),
        Box::new(crate::rules::react::no_danger::NoDanger),
        Box::new(crate::rules::react::no_danger_with_children::NoDangerWithChildren),
        Box::new(crate::rules::react::no_did_mount_set_state::NoDidMountSetState),
        Box::new(crate::rules::react::no_direct_mutation_state::NoDirectMutationState),
        Box::new(crate::rules::react::no_find_dom_node::NoFindDomNode),
        Box::new(crate::rules::react::no_is_mounted::NoIsMounted),
        Box::new(crate::rules::react::no_multi_comp::NoMultiComp),
        Box::new(crate::rules::react::no_namespace::NoNamespace),
        Box::new(crate::rules::react::no_redundant_should_component_update::NoRedundantShouldComponentUpdate),
        Box::new(crate::rules::react::no_render_return_value::NoRenderReturnValue),
        Box::new(crate::rules::react::no_set_state::NoSetState),
        Box::new(crate::rules::react::no_string_refs::NoStringRefs),
        Box::new(crate::rules::react::no_this_in_sfc::NoThisInSfc),
        Box::new(crate::rules::react::no_unescaped_entities::NoUnescapedEntities),
        Box::new(crate::rules::react::no_unknown_property::NoUnknownProperty),
        Box::new(crate::rules::react::no_unsafe::NoUnsafe),
        Box::new(crate::rules::react::no_will_update_set_state::NoWillUpdateSetState),
        Box::new(crate::rules::react::only_export_components::OnlyExportComponents),
        Box::new(crate::rules::react::prefer_es6_class::PreferEs6Class),
        Box::new(crate::rules::react::react_in_jsx_scope::ReactInJsxScope),
        Box::new(crate::rules::react::require_render_return::RequireRenderReturn),
        Box::new(crate::rules::react::rules_of_hooks::RulesOfHooks::new()),
        Box::new(crate::rules::react::self_closing_comp::SelfClosingComp),
        Box::new(crate::rules::react::state_in_constructor::StateInConstructor),
        Box::new(crate::rules::react::style_prop_object::StylePropObject),
        Box::new(crate::rules::react::void_dom_elements_no_children::VoidDomElementsNoChildren),
        Box::new(crate::rules::jsx_a11y::alt_text::AltText),
        Box::new(crate::rules::jsx_a11y::anchor_ambiguous_text::AnchorAmbiguousText),
        Box::new(crate::rules::jsx_a11y::anchor_has_content::AnchorHasContent),
        Box::new(crate::rules::jsx_a11y::anchor_is_valid::AnchorIsValid),
        Box::new(crate::rules::jsx_a11y::aria_props::AriaProps),
        Box::new(crate::rules::jsx_a11y::aria_proptypes::AriaProptypes),
        Box::new(crate::rules::jsx_a11y::aria_role::AriaRole),
        Box::new(crate::rules::jsx_a11y::aria_unsupported_elements::AriaUnsupportedElements),
        Box::new(crate::rules::jsx_a11y::autocomplete_valid::AutocompleteValid),
        Box::new(crate::rules::jsx_a11y::click_events_have_key_events::ClickEventsHaveKeyEvents),
        Box::new(crate::rules::jsx_a11y::heading_has_content::HeadingHasContent),
        Box::new(crate::rules::jsx_a11y::html_has_lang::HtmlHasLang),
        Box::new(crate::rules::jsx_a11y::iframe_has_title::IframeHasTitle),
        Box::new(crate::rules::jsx_a11y::img_redundant_alt::ImgRedundantAlt),
        Box::new(crate::rules::jsx_a11y::label_has_associated_control::LabelHasAssociatedControl),
        Box::new(crate::rules::jsx_a11y::lang::Lang),
        Box::new(crate::rules::jsx_a11y::media_has_caption::MediaHasCaption),
        Box::new(crate::rules::jsx_a11y::mouse_events_have_key_events::MouseEventsHaveKeyEvents),
        Box::new(crate::rules::jsx_a11y::no_access_key::NoAccessKey),
        Box::new(crate::rules::jsx_a11y::no_aria_hidden_on_focusable::NoAriaHiddenOnFocusable),
        Box::new(crate::rules::jsx_a11y::no_autofocus::NoAutofocus),
        Box::new(crate::rules::jsx_a11y::no_distracting_elements::NoDistractingElements),
        Box::new(crate::rules::jsx_a11y::no_noninteractive_tabindex::NoNoninteractiveTabindex),
        Box::new(crate::rules::jsx_a11y::no_redundant_roles::NoRedundantRoles),
        Box::new(
            crate::rules::jsx_a11y::no_static_element_interactions::NoStaticElementInteractions,
        ),
        Box::new(crate::rules::jsx_a11y::prefer_tag_over_role::PreferTagOverRole),
        Box::new(crate::rules::jsx_a11y::role_has_required_aria_props::RoleHasRequiredAriaProps),
        Box::new(crate::rules::jsx_a11y::role_supports_aria_props::RoleSupportAriaProps),
        Box::new(crate::rules::jsx_a11y::scope::Scope),
        Box::new(crate::rules::jsx_a11y::tabindex_no_positive::TabindexNoPositive),
        Box::new(crate::rules::react_perf::jsx_no_jsx_as_prop::JsxNoJsxAsProp),
        Box::new(crate::rules::react_perf::jsx_no_new_array_as_prop::JsxNoNewArrayAsProp),
        Box::new(crate::rules::react_perf::jsx_no_new_function_as_prop::JsxNoNewFunctionAsProp),
        Box::new(crate::rules::react_perf::jsx_no_new_object_as_prop::JsxNoNewObjectAsProp),
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
            "react plugin should provide at least one rule"
        );
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 87, "react should have 87 rules");
    }
}
