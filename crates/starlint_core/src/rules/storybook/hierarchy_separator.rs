//! Rule: `storybook/hierarchy-separator`
//!
//! Deprecated hierarchy separator in title property.
//! Checks for `|` in title strings (should use `/` instead).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/hierarchy-separator";

/// Deprecated hierarchy separator in title property.
#[derive(Debug)]
pub struct HierarchySeparator;

impl NativeRule for HierarchySeparator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Deprecated hierarchy separator in title property".to_owned(),
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

        // Find title property in default export meta
        // Look for patterns like `title: 'Components|Button'` or `title: "Components|Button"`
        let title_patterns = ["title: '", "title: \"", "title:'", "title:\""];

        for pattern in &title_patterns {
            let mut search_pos = 0;
            while let Some(pos) = source.get(search_pos..).and_then(|s| s.find(pattern)) {
                let abs_pos = search_pos.saturating_add(pos);
                let value_start = abs_pos.saturating_add(pattern.len());
                let quote_char = pattern.chars().last().unwrap_or('\'');

                // Find the closing quote
                let remaining = source.get(value_start..).unwrap_or_default();
                let Some(close_pos) = remaining.find(quote_char) else {
                    search_pos = value_start;
                    continue;
                };

                let title_value = remaining.get(..close_pos).unwrap_or_default();

                if title_value.contains('|') {
                    let start = u32::try_from(value_start).unwrap_or(0);
                    let end = start.saturating_add(u32::try_from(close_pos).unwrap_or(0));
                    let fixed_title = title_value.replace('|', "/");
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Use `/` instead of `|` as hierarchy separator in title"
                            .to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: Some("Replace `|` with `/`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Replace `|` with `/`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(start, end),
                                replacement: fixed_title,
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }

                search_pos = value_start.saturating_add(close_pos).saturating_add(1);
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(HierarchySeparator)];
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
    fn test_flags_pipe_separator() {
        let diags = lint("export default { title: 'Components|Button' };");
        assert_eq!(diags.len(), 1, "should flag | in title");
    }

    #[test]
    fn test_allows_slash_separator() {
        let diags = lint("export default { title: 'Components/Button' };");
        assert!(diags.is_empty(), "should allow / in title");
    }

    #[test]
    fn test_allows_no_separator() {
        let diags = lint("export default { title: 'Button' };");
        assert!(diags.is_empty(), "should allow title without separator");
    }
}
