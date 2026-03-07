//! Next.js-specific lint rules.
//!
//! Rules are prefixed with `nextjs/` in config and output.

pub mod google_font_display;
pub mod google_font_preconnect;
pub mod inline_script_id;
pub mod next_script_for_ga;
pub mod no_assign_module_variable;
pub mod no_async_client_component;
pub mod no_before_interactive_script_outside_document;
pub mod no_css_tags;
pub mod no_document_import_in_page;
pub mod no_duplicate_head;
pub mod no_head_element;
pub mod no_head_import_in_document;
pub mod no_html_link_for_pages;
pub mod no_img_element;
pub mod no_page_custom_font;
pub mod no_script_component_in_head;
pub mod no_styled_jsx_in_document;
pub mod no_sync_scripts;
pub mod no_title_in_document_head;
pub mod no_typos;
pub mod no_unwanted_polyfillio;

use crate::rule::NativeRule;

/// Return all Next.js rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![]
}
