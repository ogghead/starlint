//! Rule: `storybook/csf-component`
//!
//! The `component` property should be set in CSF meta.
//! Checks the default export object for a `component` property.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/csf-component";

/// The `component` property should be set in CSF meta.
#[derive(Debug)]
pub struct CsfComponent;

impl LintRule for CsfComponent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "The `component` property should be set in CSF meta".to_owned(),
            category: Category::Correctness,
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

        // Look for the object body after `export default`
        let after_default = source.get(default_pos..).unwrap_or_default();

        // Find the opening brace of the object
        let Some(brace_pos) = after_default.find('{') else {
            return;
        };

        // Find the matching closing brace (simple scan)
        let obj_start = default_pos.saturating_add(brace_pos);
        let obj_content = source.get(obj_start..).unwrap_or_default();

        // Check if `component` property exists in the meta object
        if !obj_content.contains("component") {
            let start = u32::try_from(default_pos).unwrap_or(0);
            let end = start.saturating_add(u32::try_from("export default".len()).unwrap_or(0));
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "CSF meta should include a `component` property".to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CsfComponent)];
        lint_source(source, "Button.stories.tsx", &rules)
    }

    #[test]
    fn test_flags_missing_component() {
        let diags = lint("export default { title: 'Button' };");
        assert_eq!(diags.len(), 1, "should flag meta without component");
    }

    #[test]
    fn test_allows_meta_with_component() {
        let diags = lint("export default { title: 'Button', component: Button };");
        assert!(diags.is_empty(), "should allow meta with component");
    }

    #[test]
    fn test_ignores_non_story_files() {
        let source = "export default { title: 'Button' };";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CsfComponent)];
        let diags = lint_source(source, "utils.ts", &rules);
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
