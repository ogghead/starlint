//! Rule: `storybook/use-storybook-expect`
//!
//! Use `expect` from `@storybook/test` instead of generic `expect`.
//! Checks for `expect` calls not imported from storybook.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/use-storybook-expect";

/// Use `expect` from `@storybook/test` instead of generic `expect`.
#[derive(Debug)]
pub struct UseStorybookExpect;

impl NativeRule for UseStorybookExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Use `expect` from `@storybook/test` instead of generic `expect`"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
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
                ctx.report_warning(
                    RULE_NAME,
                    "Import `expect` from `@storybook/test` instead of using generic `expect`",
                    Span::new(start, end),
                );
            }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("Button.stories.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(UseStorybookExpect)];
            traverse_and_lint(
                &parsed.program,
                &rules,
                source,
                Path::new("Button.stories.ts"),
            )
        } else {
            vec![]
        }
    }

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
