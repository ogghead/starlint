//! Rule: `storybook/default-exports`
//!
//! Story files should have a default export.
//! Scans for `export default` in `.stories.` files.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/default-exports";

/// Story files should have a default export.
#[derive(Debug)]
pub struct DefaultExports;

impl NativeRule for DefaultExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Story files should have a default export".to_owned(),
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

        if !source.contains("export default") {
            // Report at the beginning of the file
            ctx.report_warning(
                RULE_NAME,
                "Story files should have a default export (CSF meta)",
                Span::new(0, 0),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DefaultExports)];
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
        let allocator = Allocator::default();
        let source = "export const foo = 1;";
        let diags = if let Ok(parsed) = parse_file(&allocator, source, Path::new("utils.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DefaultExports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("utils.ts"))
        } else {
            vec![]
        };
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
