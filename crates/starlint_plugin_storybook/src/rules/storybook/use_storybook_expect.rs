//! Rule: `storybook/use-storybook-expect`
//!
//! Use `expect` from `@storybook/test` instead of generic `expect`.
//! Checks for `expect` calls not imported from storybook.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/use-storybook-expect";

/// Use `expect` from `@storybook/test` instead of generic `expect`.
#[derive(Debug)]
pub struct UseStorybookExpect;

impl LintRule for UseStorybookExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Use `expect` from `@storybook/test` instead of generic `expect`"
                .to_owned(),
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

        // Check if `expect` is used
        if !source.contains("expect(") {
            return;
        }

        // Check if `expect` is imported from storybook
        let has_storybook_expect = source.contains("@storybook/test")
            || source.contains("@storybook/jest")
            || source.contains("@storybook/expect");

        if !has_storybook_expect {
            // Find the first `expect(` usage and flag it
            if let Some(pos) = source.find("expect(") {
                let start = u32::try_from(pos).unwrap_or(0);
                let end = start.saturating_add(u32::try_from("expect(".len()).unwrap_or(0));
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message:
                        "Import `expect` from `@storybook/test` instead of using generic `expect`"
                            .to_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(UseStorybookExpect, "Button.stories.ts");

    #[test]
    fn test_flags_generic_expect() {
        let diags = lint(
            "export default {}; export const Test = { play: async () => { expect(1).toBe(1); } };",
        );
        assert_eq!(diags.len(), 1, "should flag generic expect");
    }

    #[test]
    fn test_allows_storybook_expect() {
        let diags = lint(
            "import { expect } from '@storybook/test'; export default {}; export const Test = { play: async () => { expect(1).toBe(1); } };",
        );
        assert!(diags.is_empty(), "should allow storybook expect");
    }

    #[test]
    fn test_ignores_no_expect() {
        let diags = lint("export default {}; export const Primary = {};");
        assert!(diags.is_empty(), "should ignore files without expect");
    }
}
