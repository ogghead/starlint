//! Nextjs lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! nextjs rules.

pub mod rules;

use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

/// Create the nextjs plugin with all its rules.
#[must_use]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LintRulePlugin::new(all_rules()))
}

/// Return all nextjs lint rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(crate::rules::nextjs::google_font_display::GoogleFontDisplay),
        Box::new(crate::rules::nextjs::google_font_preconnect::GoogleFontPreconnect),
        Box::new(crate::rules::nextjs::inline_script_id::InlineScriptId),
        Box::new(crate::rules::nextjs::next_script_for_ga::NextScriptForGa),
        Box::new(crate::rules::nextjs::no_assign_module_variable::NoAssignModuleVariable),
        Box::new(crate::rules::nextjs::no_async_client_component::NoAsyncClientComponent),
        Box::new(crate::rules::nextjs::no_before_interactive_script_outside_document::NoBeforeInteractiveScriptOutsideDocument),
        Box::new(crate::rules::nextjs::no_css_tags::NoCssTags),
        Box::new(crate::rules::nextjs::no_document_import_in_page::NoDocumentImportInPage),
        Box::new(crate::rules::nextjs::no_duplicate_head::NoDuplicateHead),
        Box::new(crate::rules::nextjs::no_head_element::NoHeadElement),
        Box::new(crate::rules::nextjs::no_head_import_in_document::NoHeadImportInDocument),
        Box::new(crate::rules::nextjs::no_html_link_for_pages::NoHtmlLinkForPages),
        Box::new(crate::rules::nextjs::no_img_element::NoImgElement),
        Box::new(crate::rules::nextjs::no_page_custom_font::NoPageCustomFont),
        Box::new(crate::rules::nextjs::no_script_component_in_head::NoScriptComponentInHead),
        Box::new(crate::rules::nextjs::no_styled_jsx_in_document::NoStyledJsxInDocument),
        Box::new(crate::rules::nextjs::no_sync_scripts::NoSyncScripts),
        Box::new(crate::rules::nextjs::no_title_in_document_head::NoTitleInDocumentHead),
        Box::new(crate::rules::nextjs::no_typos::NoTypos),
        Box::new(crate::rules::nextjs::no_unwanted_polyfillio::NoUnwantedPolyfillio),
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
            "nextjs plugin should provide at least one rule"
        );
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 21, "nextjs should have 21 rules");
    }
}
