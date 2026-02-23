//! Rule: `storybook/default-exports`
//!
//! Story files should have a default export.
//! Scans for `export default` in `.stories.` files.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/default-exports";

/// Story files should have a default export.
#[derive(Debug)]
pub struct DefaultExports;

impl LintRule for DefaultExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Story files should have a default export".to_owned(),
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

        if !source.contains("export default") {
            // Report at the beginning of the file
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Story files should have a default export (CSF meta)".to_owned(),
                span: Span::new(0, 0),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(DefaultExports)];
        lint_source(source, "Button.stories.tsx", &rules)
    }

    #[test]
    fn test_flags_missing_default_export() {
        let diags = lint("export const Primary = {};");
        assert_eq!(diags.len(), 1, "should flag missing default export");
    }

    #[test]
    fn test_allows_default_export() {
        let diags = lint("export default { title: 'Button' }; export const Primary = {};");
        assert!(diags.is_empty(), "should allow file with default export");
    }

    #[test]
    fn test_ignores_non_story_files() {
        let source = "export const foo = 1;";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(DefaultExports)];
        let diags = lint_source(source, "utils.ts", &rules);
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
