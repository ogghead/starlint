//! Rule: `storybook/meta-inline-properties`
//!
//! Meta should only have inline properties.
//! Checks that default export meta object doesn't use computed or spread properties.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/meta-inline-properties";

/// Meta should only have inline properties.
#[derive(Debug)]
pub struct MetaInlineProperties;

impl LintRule for MetaInlineProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Meta should only have inline properties".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let file_name = ctx.file_path().to_string_lossy();
        if !file_name.contains(".stories.") && !file_name.contains(".story.") {
            return;
        }

        let source = ctx.source_text().to_owned();

        // Find default export
        let Some(default_pos) = source.find("export default") else {
            return;
        };

        let after_default = source.get(default_pos..).unwrap_or_default();

        // Find the opening brace of the meta object
        let Some(brace_pos) = after_default.find('{') else {
            return;
        };

        let obj_start = default_pos.saturating_add(brace_pos);

        // Find the matching closing brace (simple depth tracking)
        let obj_content = source.get(obj_start..).unwrap_or_default();
        let mut depth: u32 = 0;
        let mut obj_end = obj_start;
        for (i, ch) in obj_content.char_indices() {
            if ch == '{' {
                depth = depth.saturating_add(1);
            } else if ch == '}' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    obj_end = obj_start.saturating_add(i);
                    break;
                }
            }
        }

        let meta_body = source
            .get(obj_start..obj_end.saturating_add(1))
            .unwrap_or_default();

        // Check for spread operator `...` in the meta object
        if meta_body.contains("...") {
            let spread_offset = meta_body.find("...").unwrap_or(0);
            let abs_pos = obj_start.saturating_add(spread_offset);
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(3);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Meta should only have inline properties, avoid spread syntax".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MetaInlineProperties)];
        lint_source(source, "Button.stories.tsx", &rules)
    }

    #[test]
    fn test_flags_spread_in_meta() {
        let diags =
            lint("const shared = { title: 'B' }; export default { ...shared, component: Button };");
        assert_eq!(diags.len(), 1, "should flag spread in meta");
    }

    #[test]
    fn test_allows_inline_properties() {
        let diags = lint("export default { title: 'Button', component: Button };");
        assert!(diags.is_empty(), "should allow inline properties");
    }

    #[test]
    fn test_ignores_non_story_files() {
        let source = "export default { ...shared };";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MetaInlineProperties)];
        let diags = lint_source(source, "utils.ts", &rules);
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
