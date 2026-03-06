//! Rule: `storybook/story-exports`
//!
//! A story file must contain at least one story export.
//! Checks for named exports in `.stories.` files.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/story-exports";

/// A story file must contain at least one story export.
#[derive(Debug)]
pub struct StoryExports;

impl NativeRule for StoryExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "A story file must contain at least one story export".to_owned(),
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

        // Check for named exports (not default)
        // Patterns: `export const`, `export let`, `export function`, `export class`
        let has_named_export = source.contains("export const ")
            || source.contains("export let ")
            || source.contains("export function ")
            || source.contains("export class ");

        // Also check for `export {` re-exports
        let has_reexport = source.contains("export {");

        if !has_named_export && !has_reexport {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Story files must contain at least one named story export".to_owned(),
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("Button.stories.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(StoryExports)];
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
    fn test_flags_no_story_exports() {
        let diags = lint("export default { title: 'Button' };");
        assert_eq!(diags.len(), 1, "should flag file with no story exports");
    }

    #[test]
    fn test_allows_named_export() {
        let diags = lint("export default { title: 'Button' }; export const Primary = {};");
        assert!(diags.is_empty(), "should allow file with story exports");
    }

    #[test]
    fn test_ignores_non_story_files() {
        let allocator = Allocator::default();
        let source = "export default {};";
        let diags = if let Ok(parsed) = parse_file(&allocator, source, Path::new("utils.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(StoryExports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("utils.ts"))
        } else {
            vec![]
        };
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
