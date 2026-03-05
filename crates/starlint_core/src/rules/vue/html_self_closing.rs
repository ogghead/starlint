//! Rule: `vue/html-self-closing`
//!
//! Enforce self-closing on components without content.
//! Scans for patterns like `<MyComponent></MyComponent>` that should be
//! `<MyComponent />`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/html-self-closing";

/// Enforce self-closing on components without content.
#[derive(Debug)]
pub struct HtmlSelfClosing;

impl NativeRule for HtmlSelfClosing {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce self-closing on components without content".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Look for patterns: `<TagName></TagName>` (empty content between open and close)
        // Focus on PascalCase tags (Vue components)
        let mut pos = 0;
        while let Some(open) = source.get(pos..).and_then(|s| s.find("</")) {
            let abs_close_start = pos.saturating_add(open);
            let after_slash = source
                .get(abs_close_start.saturating_add(2)..)
                .unwrap_or_default();

            // Extract tag name from closing tag
            let tag_end = after_slash.find('>').unwrap_or(after_slash.len());
            let tag_name = after_slash.get(..tag_end).unwrap_or_default().trim();

            // Only check PascalCase component names (uppercase first letter)
            let is_component = tag_name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase());

            if is_component && !tag_name.is_empty() {
                // Look backwards for the matching open tag: `<TagName>`
                let open_pattern = format!("<{tag_name}");
                let before = source.get(..abs_close_start).unwrap_or_default();

                if let Some(open_pos) = before.rfind(&open_pattern) {
                    // Find the end of the opening tag
                    let after_open = source.get(open_pos..).unwrap_or_default();
                    if let Some(gt_pos) = after_open.find('>') {
                        let open_end = open_pos.saturating_add(gt_pos).saturating_add(1);
                        // Check if content between open and close is empty/whitespace
                        let content = source.get(open_end..abs_close_start).unwrap_or_default();
                        if content.trim().is_empty() {
                            let start = u32::try_from(open_pos).unwrap_or(0);
                            let end = u32::try_from(
                                abs_close_start
                                    .saturating_add(3)
                                    .saturating_add(tag_name.len()),
                            )
                            .unwrap_or(0);
                            ctx.report(Diagnostic {
                                rule_name: RULE_NAME.to_owned(),
                                message: format!(
                                    "`<{tag_name}></{tag_name}>` should be self-closing: `<{tag_name} />`"
                                ),
                                span: Span::new(start, end),
                                severity: Severity::Warning,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                }
            }

            pos = abs_close_start.saturating_add(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(HtmlSelfClosing)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_component() {
        let source = r"const t = '<MyComponent></MyComponent>';";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "empty component tag should be flagged");
    }

    #[test]
    fn test_allows_self_closing() {
        let source = r"const t = '<MyComponent />';";
        let diags = lint(source);
        assert!(diags.is_empty(), "self-closing should be allowed");
    }

    #[test]
    fn test_allows_component_with_content() {
        let source = r"const t = '<MyComponent>Hello</MyComponent>';";
        let diags = lint(source);
        assert!(diags.is_empty(), "component with content should be allowed");
    }
}
