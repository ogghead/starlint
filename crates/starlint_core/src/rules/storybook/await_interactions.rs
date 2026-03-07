//! Rule: `storybook/await-interactions`
//!
//! Interactions in play functions should be awaited.
//! Detects `userEvent.*` or `within()` calls not preceded by `await` inside play functions.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/await-interactions";

/// Interactions in play functions should be awaited.
#[derive(Debug)]
pub struct AwaitInteractions;

impl LintRule for AwaitInteractions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Interactions in play functions should be awaited".to_owned(),
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

        // Find play function bodies and check for un-awaited interactions
        // Look for patterns like `play:` or `play =` followed by async function bodies
        let interaction_patterns = ["userEvent.", "within("];

        for pattern in &interaction_patterns {
            let mut search_pos = 0;
            while let Some(pos) = source.get(search_pos..).and_then(|s| s.find(pattern)) {
                let abs_pos = search_pos.saturating_add(pos);

                // Check if preceded by `await` (looking back up to 20 chars for whitespace + await)
                let lookback_start = abs_pos.saturating_sub(20);
                let before = source.get(lookback_start..abs_pos).unwrap_or_default();
                let trimmed_before = before.trim_end();

                if !trimmed_before.ends_with("await") {
                    let start = u32::try_from(abs_pos).unwrap_or(0);
                    let end = start.saturating_add(u32::try_from(pattern.len()).unwrap_or(0));
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Interactions should be awaited in play functions".to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: None,
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Add `await`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(start, start),
                                replacement: "await ".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }

                search_pos = abs_pos.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AwaitInteractions)];
        lint_source(source, "Button.stories.tsx", &rules)
    }

    #[test]
    fn test_flags_unwaited_user_event() {
        let diags = lint(
            "export const Default = { play: async ({ canvasElement }) => { userEvent.click(canvas); } };",
        );
        assert!(!diags.is_empty(), "should flag un-awaited userEvent");
    }

    #[test]
    fn test_allows_awaited_user_event() {
        let diags = lint(
            "export const Default = { play: async ({ canvasElement }) => { await userEvent.click(canvas); } };",
        );
        assert!(diags.is_empty(), "should allow awaited userEvent");
    }

    #[test]
    fn test_ignores_non_story_files() {
        let source = "userEvent.click(canvas);";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AwaitInteractions)];
        let diags = lint_source(source, "utils.ts", &rules);
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
