//! Rule: `storybook/await-interactions`
//!
//! Interactions in play functions should be awaited.
//! Detects `userEvent.*` or `within()` calls not preceded by `await` inside play functions.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/await-interactions";

/// Interactions in play functions should be awaited.
#[derive(Debug)]
pub struct AwaitInteractions;

impl NativeRule for AwaitInteractions {
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

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
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
                            kind: FixKind::SuggestionFix,
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("Button.stories.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AwaitInteractions)];
            traverse_and_lint(
                &parsed.program,
                &rules,
                source,
                Path::new("Button.stories.tsx"),
            )
        } else {
            vec![]
        }
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
        let allocator = Allocator::default();
        let source = "userEvent.click(canvas);";
        let diags = if let Ok(parsed) = parse_file(&allocator, source, Path::new("utils.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AwaitInteractions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("utils.ts"))
        } else {
            vec![]
        };
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
