//! Rule: `storybook/no-redundant-story-name`
//!
//! A story should not have a redundant name property.
//! Checks for `name:` property on story exports where the name matches the export name.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/no-redundant-story-name";

/// A story should not have a redundant name property.
#[derive(Debug)]
pub struct NoRedundantStoryName;

impl NativeRule for NoRedundantStoryName {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "A story should not have a redundant name property".to_owned(),
            category: Category::Style,
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

        // Find named exports like `export const Primary = { name: 'Primary' }`
        // Pattern: `export const <Name> = {` ... `name: '<Name>'`
        let export_pattern = "export const ";
        let mut search_pos = 0;
        while let Some(pos) = source
            .get(search_pos..)
            .and_then(|s| s.find(export_pattern))
        {
            let abs_pos = search_pos.saturating_add(pos);
            let after_export = source
                .get(abs_pos.saturating_add(export_pattern.len())..)
                .unwrap_or_default();

            // Get the export name (up to first space or `=` or `:`)
            let name_end = after_export.find([' ', '=', ':']).unwrap_or(0);
            let export_name = after_export.get(..name_end).unwrap_or_default().trim();

            if export_name.is_empty() {
                search_pos = abs_pos.saturating_add(1);
                continue;
            }

            // Look for the object body and find `name:` property
            let obj_region = after_export.get(name_end..).unwrap_or_default();
            let Some(brace_pos) = obj_region.find('{') else {
                search_pos = abs_pos.saturating_add(1);
                continue;
            };

            let obj_body = obj_region.get(brace_pos..).unwrap_or_default();

            // Look for name: 'ExportName' or name: "ExportName"
            for quote in ['\'', '"'] {
                let name_pattern = format!("name: {quote}{export_name}{quote}");
                if obj_body.contains(name_pattern.as_str()) {
                    let name_in_obj = obj_body.find(name_pattern.as_str()).unwrap_or(0);
                    let abs_name_pos = abs_pos
                        .saturating_add(export_pattern.len())
                        .saturating_add(name_end)
                        .saturating_add(brace_pos)
                        .saturating_add(name_in_obj);
                    let start = u32::try_from(abs_name_pos).unwrap_or(0);
                    let end = start.saturating_add(u32::try_from(name_pattern.len()).unwrap_or(0));

                    // Extend removal span to include surrounding comma and whitespace
                    let fix_start = start;
                    let mut fix_end = end;
                    let fix_end_usize = usize::try_from(fix_end).unwrap_or(0);
                    let after = source.get(fix_end_usize..).unwrap_or_default();
                    // Skip trailing whitespace and comma
                    let extra = after
                        .chars()
                        .take_while(|c| *c == ' ' || *c == ',' || *c == '\n' || *c == '\r')
                        .count();
                    fix_end = fix_end.saturating_add(u32::try_from(extra).unwrap_or(0));

                    // Also try to consume leading comma+whitespace if the property
                    // was not the first one (i.e. preceded by `, `)
                    let fix_start_usize = usize::try_from(fix_start).unwrap_or(0);
                    let before = source.get(..fix_start_usize).unwrap_or_default();
                    let leading = before
                        .chars()
                        .rev()
                        .take_while(|c| *c == ' ' || *c == ',')
                        .count();
                    let actual_start =
                        fix_start.saturating_sub(u32::try_from(leading).unwrap_or(0));

                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Story name property is redundant when it matches the export name"
                            .to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: Some("Remove the redundant `name` property".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Remove the redundant `name` property".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(actual_start, fix_end),
                                replacement: String::new(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }

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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRedundantStoryName)];
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
    fn test_flags_redundant_name() {
        let diags = lint("export default {}; export const Primary = { name: 'Primary' };");
        assert_eq!(diags.len(), 1, "should flag redundant name property");
    }

    #[test]
    fn test_allows_different_name() {
        let diags = lint("export default {}; export const Primary = { name: 'Main Button' };");
        assert!(diags.is_empty(), "should allow different name");
    }

    #[test]
    fn test_allows_no_name() {
        let diags = lint("export default {}; export const Primary = {};");
        assert!(diags.is_empty(), "should allow story without name prop");
    }
}
