//! Rule: `vue/no-child-content`
//!
//! Forbid using `v-html` or `v-text` directives on elements that also have
//! child content. When both are present, the directive overwrites the children.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-child-content";

/// Forbid `v-html`/`v-text` on elements with children.
#[derive(Debug)]
pub struct NoChildContent;

impl LintRule for NoChildContent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `v-html`/`v-text` on elements with child content".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        for directive in &["v-html", "v-text"] {
            let mut search_pos = 0;
            while let Some(offset) = source.get(search_pos..).and_then(|s| s.find(directive)) {
                let abs_pos = search_pos.saturating_add(offset);

                // Find the opening tag that contains this directive — scan backward for `<`
                let before = source.get(..abs_pos).unwrap_or_default();
                let Some(tag_open) = before.rfind('<') else {
                    search_pos = abs_pos.saturating_add(directive.len());
                    continue;
                };

                // Find the tag name
                let tag_content = source
                    .get(tag_open.saturating_add(1)..abs_pos)
                    .unwrap_or_default();
                let tag_name = tag_content.split_whitespace().next().unwrap_or_default();

                if tag_name.is_empty() || tag_name.starts_with('/') {
                    search_pos = abs_pos.saturating_add(directive.len());
                    continue;
                }

                // Find the closing `>` of the opening tag
                let after = source.get(abs_pos..).unwrap_or_default();
                let Some(tag_close) = after.find('>') else {
                    search_pos = abs_pos.saturating_add(directive.len());
                    continue;
                };

                // Check if self-closing
                let tag_end_area = after.get(..tag_close).unwrap_or_default();
                if tag_end_area.ends_with('/') {
                    search_pos = abs_pos.saturating_add(tag_close).saturating_add(1);
                    continue;
                }

                // Find the matching closing tag
                let closing_tag = format!("</{tag_name}>");
                let content_start = abs_pos.saturating_add(tag_close).saturating_add(1);
                let rest = source.get(content_start..).unwrap_or_default();

                if let Some(close_pos) = rest.find(&closing_tag) {
                    let inner = rest.get(..close_pos).unwrap_or_default();
                    if !inner.trim().is_empty() {
                        let start = u32::try_from(abs_pos).unwrap_or(0);
                        let end = start.saturating_add(u32::try_from(directive.len()).unwrap_or(0));
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: format!(
                                "Element with `{directive}` should not have child content — the directive will overwrite it"
                            ),
                            span: Span::new(start, end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }

                search_pos = abs_pos.saturating_add(directive.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoChildContent);

    #[test]
    fn test_flags_v_html_with_children() {
        let source = r#"const t = '<div v-html="raw">child text</div>';"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "v-html with child content should be flagged"
        );
    }

    #[test]
    fn test_allows_v_html_empty() {
        let source = r#"const t = '<div v-html="raw"></div>';"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "v-html without children should be allowed"
        );
    }

    #[test]
    fn test_flags_v_text_with_children() {
        let source = r#"const t = '<span v-text="msg">old text</span>';"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "v-text with child content should be flagged"
        );
    }
}
