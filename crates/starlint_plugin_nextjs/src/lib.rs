//! Nextjs lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! nextjs rules.

pub mod rules;

starlint_rule_framework::declare_plugin! {
    name: "nextjs",
    rules: [
        crate::rules::nextjs::google_font_display::GoogleFontDisplay,
        crate::rules::nextjs::google_font_preconnect::GoogleFontPreconnect,
        crate::rules::nextjs::inline_script_id::InlineScriptId,
        crate::rules::nextjs::next_script_for_ga::NextScriptForGa,
        crate::rules::nextjs::no_assign_module_variable::NoAssignModuleVariable,
        crate::rules::nextjs::no_async_client_component::NoAsyncClientComponent,
        crate::rules::nextjs::no_before_interactive_script_outside_document::NoBeforeInteractiveScriptOutsideDocument,
        crate::rules::nextjs::no_css_tags::NoCssTags,
        crate::rules::nextjs::no_document_import_in_page::NoDocumentImportInPage,
        crate::rules::nextjs::no_duplicate_head::NoDuplicateHead,
        crate::rules::nextjs::no_head_element::NoHeadElement,
        crate::rules::nextjs::no_head_import_in_document::NoHeadImportInDocument,
        crate::rules::nextjs::no_html_link_for_pages::NoHtmlLinkForPages,
        crate::rules::nextjs::no_img_element::NoImgElement,
        crate::rules::nextjs::no_page_custom_font::NoPageCustomFont,
        crate::rules::nextjs::no_script_component_in_head::NoScriptComponentInHead,
        crate::rules::nextjs::no_styled_jsx_in_document::NoStyledJsxInDocument,
        crate::rules::nextjs::no_sync_scripts::NoSyncScripts,
        crate::rules::nextjs::no_title_in_document_head::NoTitleInDocumentHead,
        crate::rules::nextjs::no_typos::NoTypos,
        crate::rules::nextjs::no_unwanted_polyfillio::NoUnwantedPolyfillio,
    ]
}
