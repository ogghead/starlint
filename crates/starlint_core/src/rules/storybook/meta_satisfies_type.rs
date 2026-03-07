//! Rule: `storybook/meta-satisfies-type`
//!
//! Meta should use `satisfies Meta` for type safety.
//! Checks for `export default { ... } satisfies Meta` vs `export default { ... }` without `satisfies`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/meta-satisfies-type";

/// Meta should use `satisfies Meta` for type safety.
#[derive(Debug)]
pub struct MetaSatisfiesType;

impl LintRule for MetaSatisfiesType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Meta should use `satisfies Meta` for type safety".to_owned(),
            category: Category::Suggestion,
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

        // Only check TypeScript files
        let file_ext = ctx
            .file_path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if file_ext != "ts" && file_ext != "tsx" {
            return;
        }

        // Find default export
        let Some(default_pos) = source.find("export default") else {
            return;
        };

        // Check if `satisfies` appears after the default export object
        let after_default = source.get(default_pos..).unwrap_or_default();
        if !after_default.contains("satisfies") {
            let start = u32::try_from(default_pos).unwrap_or(0);
            let end = start.saturating_add(u32::try_from("export default".len()).unwrap_or(0));
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Meta should use `satisfies Meta` for type safety".to_owned(),
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
    use crate::lint_rule::lint_source;
    fn lint_with_path(
        source: &str,
        path: &str,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MetaSatisfiesType)];
        lint_source(source, path, &rules)
    }

    #[test]
    fn test_flags_missing_satisfies() {
        let diags = lint_with_path(
            "export default { title: 'Button', component: Button };",
            "Button.stories.ts",
        );
        assert_eq!(diags.len(), 1, "should flag meta without satisfies");
    }

    #[test]
    fn test_allows_satisfies_meta() {
        let diags = lint_with_path(
            "export default { title: 'Button', component: Button } satisfies Meta;",
            "Button.stories.ts",
        );
        assert!(diags.is_empty(), "should allow meta with satisfies");
    }

    #[test]
    fn test_ignores_js_files() {
        let diags = lint_with_path("export default { title: 'Button' };", "Button.stories.js");
        assert!(
            diags.is_empty(),
            "should ignore JS files (satisfies is TS-only)"
        );
    }
}
