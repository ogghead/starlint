//! Vue-specific lint rules.
//!
//! Rules are prefixed with `vue/` in config and output.

pub mod component_definition_name_casing;
pub mod custom_event_name_casing;
pub mod html_closing_bracket_newline;
pub mod html_self_closing;
pub mod no_arrow_functions_in_watch;
pub mod no_async_in_computed_properties;
pub mod no_child_content;
pub mod no_component_options_typo;
pub mod no_dupe_keys;
pub mod no_expose_after_await;
pub mod no_lifecycle_after_await;
pub mod no_ref_object_reactivity_loss;
pub mod no_reserved_component_names;
pub mod no_setup_props_reactivity_loss;
pub mod no_watch_after_await;
pub mod prefer_define_options;
pub mod require_prop_comment;

use crate::rule::NativeRule;

/// Return all Vue rules.
#[must_use]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![Box::new(custom_event_name_casing::CustomEventNameCasing)]
}
