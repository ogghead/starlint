//! Rule: `vue/prefer-define-options`
//!
//! Prefer `defineOptions()` over the options API export for setting component
//! options in `<script setup>`. The `defineOptions()` macro is the idiomatic
//! way to set component metadata in Vue 3.3+.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/prefer-define-options";

/// Options that can be set via `defineOptions()`.
const DEFINE_OPTIONS_KEYS: &[&str] = &["name:", "inheritAttrs:"];

/// Prefer `defineOptions()` over options API export.
#[derive(Debug)]
pub struct PreferDefineOptions;

impl LintRule for PreferDefineOptions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `defineOptions()` over options API export for component metadata"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Check if file uses `defineOptions` already — if so, skip
        if source.contains("defineOptions") {
            return;
        }

        // Look for `export default {` with option keys that could use defineOptions
        let Some(export_pos) = source.find("export default {") else {
            return;
        };

        let after_export = source.get(export_pos..).unwrap_or_default();

        for key in DEFINE_OPTIONS_KEYS {
            if after_export.contains(key) {
                let start = u32::try_from(export_pos).unwrap_or(0);
                let end = start.saturating_add(16); // "export default {" length
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Consider using `defineOptions()` instead of `export default` for component option `{option}`",
                        option = key.trim_end_matches(':')
                    ),
                    span: Span::new(start, end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                // Only report once per file
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(PreferDefineOptions);

    #[test]
    fn test_flags_export_default_with_name() {
        let source = r#"export default { name: "MyComponent", setup() {} };"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "export default with name should be flagged");
    }

    #[test]
    fn test_allows_define_options() {
        let source = r#"defineOptions({ name: "MyComponent" });"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "defineOptions usage should be allowed");
    }

    #[test]
    fn test_allows_export_without_options_keys() {
        let source = r"export default { setup() { return {}; } };";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "export default without option keys should be allowed"
        );
    }
}
