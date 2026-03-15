//! Rule: `storybook/context-in-play-function`
//!
//! Pass a context when invoking play function of another story.
//! Checks for `.play()` calls inside play functions that don't pass arguments.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/context-in-play-function";

/// Pass a context when invoking the play function of another story.
#[derive(Debug)]
pub struct ContextInPlayFunction;

impl LintRule for ContextInPlayFunction {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Pass a context when invoking play function of another story".to_owned(),
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

        // Look for `.play()` calls with empty parens (no context argument)
        let pattern = ".play()";
        let mut search_pos = 0;
        while let Some(pos) = source.get(search_pos..).and_then(|s| s.find(pattern)) {
            let abs_pos = search_pos.saturating_add(pos);
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(u32::try_from(pattern.len()).unwrap_or(0));
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Pass the context argument when calling another story's play function"
                    .to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
            search_pos = abs_pos.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_rule_framework::lint_source;
    starlint_rule_framework::lint_rule_test!(ContextInPlayFunction, "Button.stories.tsx");

    #[test]
    fn test_flags_play_without_context() {
        let diags = lint("export const Second = { play: async () => { Primary.play(); } };");
        assert_eq!(diags.len(), 1, "should flag .play() without context");
    }

    #[test]
    fn test_allows_play_with_context() {
        let diags = lint("export const Second = { play: async (ctx) => { Primary.play(ctx); } };");
        assert!(diags.is_empty(), "should allow .play(ctx)");
    }

    #[test]
    fn test_ignores_non_story_files() {
        let source = "Primary.play();";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ContextInPlayFunction)];
        let diags = lint_source(source, "utils.ts", &rules);
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
