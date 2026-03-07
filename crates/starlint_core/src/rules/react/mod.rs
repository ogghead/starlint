//! React-specific lint rules.
//!
//! Rules are prefixed with `react/` in config and output.

pub mod button_has_type;
pub mod checked_requires_onchange_or_readonly;
pub mod display_name;
pub mod exhaustive_deps;
pub mod forbid_dom_props;
pub mod forbid_elements;
pub mod forward_ref_uses_ref;
pub mod iframe_missing_sandbox;
pub mod jsx_boolean_value;
pub mod jsx_curly_brace_presence;
pub mod jsx_filename_extension;
pub mod jsx_fragments;
pub mod jsx_handler_names;
pub mod jsx_key;
pub mod jsx_max_depth;
pub mod jsx_no_comment_textnodes;
pub mod jsx_no_constructed_context_values;
pub mod jsx_no_duplicate_props;
pub mod jsx_no_script_url;
pub mod jsx_no_target_blank;
pub mod jsx_no_undef;
pub mod jsx_no_useless_fragment;
pub mod jsx_pascal_case;
pub mod jsx_props_no_spread_multi;
pub mod jsx_props_no_spreading;
pub mod no_array_index_key;
pub mod no_children_prop;
pub mod no_danger;
pub mod no_danger_with_children;
pub mod no_did_mount_set_state;
pub mod no_direct_mutation_state;
pub mod no_find_dom_node;
pub mod no_is_mounted;
pub mod no_multi_comp;
pub mod no_namespace;
pub mod no_redundant_should_component_update;
pub mod no_render_return_value;
pub mod no_set_state;
pub mod no_string_refs;
pub mod no_this_in_sfc;
pub mod no_unescaped_entities;
pub mod no_unknown_property;
pub mod no_unsafe;
pub mod no_will_update_set_state;
pub mod only_export_components;
pub mod prefer_es6_class;
pub mod react_in_jsx_scope;
pub mod require_render_return;
pub mod rules_of_hooks;
pub mod self_closing_comp;
pub mod state_in_constructor;
pub mod style_prop_object;
pub mod void_dom_elements_no_children;

use crate::rule::NativeRule;

/// Return all React rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(button_has_type::ButtonHasType),
        Box::new(checked_requires_onchange_or_readonly::CheckedRequiresOnchangeOrReadonly),
        Box::new(display_name::DisplayName),
        Box::new(exhaustive_deps::ExhaustiveDeps),
        Box::new(forbid_dom_props::ForbidDomProps),
        Box::new(forbid_elements::ForbidElements),
        Box::new(forward_ref_uses_ref::ForwardRefUsesRef),
        Box::new(iframe_missing_sandbox::IframeMissingSandbox),
        Box::new(jsx_boolean_value::JsxBooleanValue),
        Box::new(jsx_curly_brace_presence::JsxCurlyBracePresence),
        Box::new(jsx_fragments::JsxFragments),
        Box::new(jsx_handler_names::JsxHandlerNames),
        Box::new(jsx_key::JsxKey),
        Box::new(jsx_max_depth::JsxMaxDepth),
        Box::new(jsx_no_constructed_context_values::JsxNoConstructedContextValues),
        Box::new(jsx_no_duplicate_props::JsxNoDuplicateProps),
        Box::new(jsx_no_script_url::JsxNoScriptUrl),
        Box::new(jsx_no_target_blank::JsxNoTargetBlank),
        Box::new(jsx_no_undef::JsxNoUndef),
        Box::new(jsx_no_useless_fragment::JsxNoUselessFragment),
        Box::new(jsx_pascal_case::JsxPascalCase),
        Box::new(jsx_props_no_spread_multi::JsxPropsNoSpreadMulti),
        Box::new(no_array_index_key::NoArrayIndexKey),
        Box::new(no_children_prop::NoChildrenProp),
        Box::new(no_danger::NoDanger),
        Box::new(no_danger_with_children::NoDangerWithChildren),
        Box::new(no_did_mount_set_state::NoDidMountSetState),
        Box::new(no_direct_mutation_state::NoDirectMutationState),
        Box::new(no_find_dom_node::NoFindDomNode),
        Box::new(no_is_mounted::NoIsMounted),
        Box::new(no_multi_comp::NoMultiComp),
        Box::new(no_redundant_should_component_update::NoRedundantShouldComponentUpdate),
        Box::new(no_render_return_value::NoRenderReturnValue),
        Box::new(no_set_state::NoSetState),
        Box::new(no_string_refs::NoStringRefs),
        Box::new(no_unknown_property::NoUnknownProperty),
        Box::new(no_unsafe::NoUnsafe),
        Box::new(no_will_update_set_state::NoWillUpdateSetState),
        Box::new(only_export_components::OnlyExportComponents),
        Box::new(prefer_es6_class::PreferEs6Class),
        Box::new(require_render_return::RequireRenderReturn),
        Box::new(rules_of_hooks::RulesOfHooks::new()),
        Box::new(self_closing_comp::SelfClosingComp),
        Box::new(state_in_constructor::StateInConstructor),
        Box::new(style_prop_object::StylePropObject),
        Box::new(void_dom_elements_no_children::VoidDomElementsNoChildren),
    ]
}
