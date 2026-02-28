//! JSX accessibility lint rules.
//!
//! Rules are prefixed with `jsx-a11y/` in config and output.

pub mod alt_text;
pub mod anchor_ambiguous_text;
pub mod anchor_has_content;
pub mod anchor_is_valid;
pub mod aria_activedescendant_has_tabindex;
pub mod aria_props;
pub mod aria_proptypes;
pub mod aria_role;
pub mod aria_unsupported_elements;
pub mod autocomplete_valid;
pub mod click_events_have_key_events;
pub mod heading_has_content;
pub mod html_has_lang;
pub mod iframe_has_title;
pub mod img_redundant_alt;
pub mod label_has_associated_control;
pub mod lang;
pub mod media_has_caption;
pub mod mouse_events_have_key_events;
pub mod no_access_key;
pub mod no_aria_hidden_on_focusable;
pub mod no_autofocus;
pub mod no_distracting_elements;
pub mod no_noninteractive_tabindex;
pub mod no_redundant_roles;
pub mod no_static_element_interactions;
pub mod prefer_tag_over_role;
pub mod role_has_required_aria_props;
pub mod role_supports_aria_props;
pub mod scope;
pub mod tabindex_no_positive;

use crate::rule::NativeRule;

/// Return all JSX accessibility rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(alt_text::AltText),
        Box::new(anchor_ambiguous_text::AnchorAmbiguousText),
        Box::new(anchor_has_content::AnchorHasContent),
        Box::new(anchor_is_valid::AnchorIsValid),
        Box::new(aria_activedescendant_has_tabindex::AriaActivedescendantHasTabindex),
        Box::new(aria_props::AriaProps),
        Box::new(aria_proptypes::AriaProptypes),
        Box::new(aria_role::AriaRole),
        Box::new(aria_unsupported_elements::AriaUnsupportedElements),
        Box::new(autocomplete_valid::AutocompleteValid),
        Box::new(click_events_have_key_events::ClickEventsHaveKeyEvents),
        Box::new(heading_has_content::HeadingHasContent),
        Box::new(html_has_lang::HtmlHasLang),
        Box::new(iframe_has_title::IframeHasTitle),
        Box::new(img_redundant_alt::ImgRedundantAlt),
        Box::new(label_has_associated_control::LabelHasAssociatedControl),
        Box::new(lang::Lang),
        Box::new(media_has_caption::MediaHasCaption),
        Box::new(mouse_events_have_key_events::MouseEventsHaveKeyEvents),
        Box::new(no_access_key::NoAccessKey),
        Box::new(no_aria_hidden_on_focusable::NoAriaHiddenOnFocusable),
        Box::new(no_autofocus::NoAutofocus),
        Box::new(no_distracting_elements::NoDistractingElements),
        Box::new(no_noninteractive_tabindex::NoNoninteractiveTabindex),
        Box::new(no_redundant_roles::NoRedundantRoles),
        Box::new(no_static_element_interactions::NoStaticElementInteractions),
        Box::new(prefer_tag_over_role::PreferTagOverRole),
        Box::new(role_has_required_aria_props::RoleHasRequiredAriaProps),
        Box::new(role_supports_aria_props::RoleSupportAriaProps),
        Box::new(scope::Scope),
        Box::new(tabindex_no_positive::TabindexNoPositive),
    ]
}
