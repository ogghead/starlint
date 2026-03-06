//! Rule: `storybook/no-title-property-in-meta`
//!
//! Do not define a title in meta (use auto-title instead).
//! Checks the default export for a `title:` property.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/no-title-property-in-meta";

/// Do not define a title in meta (use auto-title instead).
#[derive(Debug)]
pub struct NoTitlePropertyInMeta;

impl NativeRule for NoTitlePropertyInMeta {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Do not define a title in meta — use auto-title instead".to_owned(),
            category: Category::Suggestion,
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

        // Find default export
        let Some(default_pos) = source.find("export default") else {
            return;
        };

        let after_default = source.get(default_pos..).unwrap_or_default();

        // Find the opening brace
        let Some(brace_pos) = after_default.find('{') else {
            return;
        };

        let obj_start = default_pos.saturating_add(brace_pos);
        let obj_content = source.get(obj_start..).unwrap_or_default();

        // Find the matching closing brace
        let mut depth: u32 = 0;
        let mut obj_end = obj_start;
        for (i, ch) in obj_content.char_indices() {
            if ch == '{' {
                depth = depth.saturating_add(1);
            } else if ch == '}' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    obj_end = obj_start.saturating_add(i);
                    break;
                }
            }
        }

        let meta_body = source
            .get(obj_start..obj_end.saturating_add(1))
            .unwrap_or_default();

        // Check for `title:` or `title :` in the meta object
        if meta_body.contains("title:") || meta_body.contains("title :") {
            let title_offset = meta_body
                .find("title:")
                .or_else(|| meta_body.find("title :"))
                .unwrap_or(0);
            let abs_pos = obj_start.saturating_add(title_offset);
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(u32::try_from("title:".len()).unwrap_or(0));
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not define a `title` in meta — use auto-title instead".to_owned(),
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("Button.stories.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTitlePropertyInMeta)];
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
    fn test_flags_title_in_meta() {
        let diags = lint("export default { title: 'Button', component: Button };");
        assert_eq!(diags.len(), 1, "should flag title in meta");
    }

    #[test]
    fn test_allows_meta_without_title() {
        let diags = lint("export default { component: Button };");
        assert!(diags.is_empty(), "should allow meta without title");
    }

    #[test]
    fn test_ignores_non_story_files() {
        let allocator = Allocator::default();
        let source = "export default { title: 'Hello' };";
        let diags = if let Ok(parsed) = parse_file(&allocator, source, Path::new("utils.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTitlePropertyInMeta)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("utils.ts"))
        } else {
            vec![]
        };
        assert!(diags.is_empty(), "should ignore non-story files");
    }
}
