//! Rule: `jsdoc/empty-tags`
//!
//! Enforce certain `JSDoc` tags have no content (e.g. `@abstract`, `@async`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Tags that should never have content after them.
const EMPTY_TAGS: &[&str] = &[
    "abstract",
    "async",
    "generator",
    "global",
    "hideconstructor",
    "ignore",
    "inner",
    "instance",
    "override",
    "readonly",
    "static",
];

#[derive(Debug)]
pub struct EmptyTags;

impl LintRule for EmptyTags {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/empty-tags".to_owned(),
            description: "Enforce certain JSDoc tags have no content".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        let mut pos = 0;
        while let Some(start) = source.get(pos..).and_then(|s| s.find("/**")) {
            let abs_start = pos.saturating_add(start);
            let search_from = abs_start.saturating_add(3);
            if let Some(end) = source.get(search_from..).and_then(|s| s.find("*/")) {
                let abs_end = search_from.saturating_add(end).saturating_add(2);
                let block = source.get(abs_start..abs_end).unwrap_or_default();

                // Track the byte offset of each line within the source
                let mut line_offset = abs_start;
                for line in block.lines() {
                    let trimmed = super::trim_jsdoc_line(line);
                    if let Some(after_at) = trimmed.strip_prefix('@') {
                        let tag_name = after_at.split_whitespace().next().unwrap_or_default();
                        if EMPTY_TAGS.contains(&tag_name) {
                            let rest = after_at.strip_prefix(tag_name).unwrap_or_default().trim();
                            if !rest.is_empty() {
                                let span_start = u32::try_from(abs_start).unwrap_or(0);
                                let span_end = u32::try_from(abs_end).unwrap_or(span_start);

                                // Find the `@tagname` in this line within the source
                                let tag_needle = format!("@{tag_name}");
                                let line_source = source.get(line_offset..).unwrap_or_default();
                                if let Some(at_pos) = line_source.find(&tag_needle) {
                                    // Content starts after `@tagname`
                                    let content_start = line_offset
                                        .saturating_add(at_pos)
                                        .saturating_add(tag_needle.len());
                                    // Content ends at end of line (before newline)
                                    // but we want to stop before `*/` if on same line
                                    let line_end = line_offset.saturating_add(line.len());
                                    let region =
                                        source.get(content_start..line_end).unwrap_or_default();
                                    // Remove everything up to `*/` or end of line
                                    let end_trim = region.find("*/").unwrap_or(region.len());
                                    // Trim trailing whitespace before `*/`
                                    let to_remove = region.get(..end_trim).unwrap_or_default();
                                    if !to_remove.trim().is_empty() {
                                        let fix_start = u32::try_from(content_start).unwrap_or(0);
                                        let fix_end = fix_start.saturating_add(
                                            u32::try_from(to_remove.len()).unwrap_or(0),
                                        );
                                        ctx.report(Diagnostic {
                                            rule_name: "jsdoc/empty-tags".to_owned(),
                                            message: format!(
                                                "`@{tag_name}` tag should not have content"
                                            ),
                                            span: Span::new(span_start, span_end),
                                            severity: Severity::Warning,
                                            help: Some(format!(
                                                "Remove content after `@{tag_name}`"
                                            )),
                                            fix: Some(Fix {
                                                kind: FixKind::SafeFix,
                                                message: format!(
                                                    "Remove content after `@{tag_name}`"
                                                ),
                                                edits: vec![Edit {
                                                    span: Span::new(fix_start, fix_end),
                                                    replacement: String::new(),
                                                }],
                                                is_snippet: false,
                                            }),
                                            labels: vec![],
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Advance past this line + newline character
                    line_offset = line_offset.saturating_add(line.len()).saturating_add(1);
                }

                pos = abs_end;
            } else {
                break;
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(EmptyTags)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_abstract_with_content() {
        let source = "/** @abstract some content */\nclass Foo {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_abstract_without_content() {
        let source = "/** @abstract */\nclass Foo {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_async_with_content() {
        let source = "/** @async yes */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }
}
