//! Rule: `storybook/context-in-play-function`
//!
//! Pass a context when invoking play function of another story.
//! Checks for `.play()` calls inside play functions that don't pass arguments.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/context-in-play-function";

/// Pass a context when invoking the play function of another story.
#[derive(Debug)]
pub struct ContextInPlayFunction;

impl NativeRule for ContextInPlayFunction {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Pass a context when invoking play function of another story".to_owned(),
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

        // Look for `.play()` calls with empty parens (no context argument)
        let pattern = ".play()";
        let mut search_pos = 0;
        while let Some(pos) = source.get(search_pos..).and_then(|s| s.find(pattern)) {
            let abs_pos = search_pos.saturating_add(pos);
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(u32::try_from(pattern.len()).unwrap_or(0));
            ctx.report_warning(
                RULE_NAME,
                "Pass the context argument when calling another story's play function",
                Span::new(start, end),
            );
            search_pos = abs_pos.saturating_add(1);
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ContextInPlayFunction)];
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
        let allocator = Allocator::default();
        let source = "Primary.play();";
        let diags = if let Ok(parsed) = parse_file(&allocator, source, Path::new("utils.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ContextInPlayFunction)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("utils.ts"))
        } else {
            vec![]
        };
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
