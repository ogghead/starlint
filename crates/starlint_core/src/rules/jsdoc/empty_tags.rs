//! Rule: `jsdoc/empty-tags`
//!
//! Enforce certain `JSDoc` tags have no content (e.g. `@abstract`, `@async`).

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for EmptyTags {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/empty-tags".to_owned(),
            description: "Enforce certain JSDoc tags have no content".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        let mut pos = 0;
        while let Some(start) = source.get(pos..).and_then(|s| s.find("/**")) {
            let abs_start = pos.saturating_add(start);
            let search_from = abs_start.saturating_add(3);
            if let Some(end) = source.get(search_from..).and_then(|s| s.find("*/")) {
                let abs_end = search_from.saturating_add(end).saturating_add(2);
                let block = source.get(abs_start..abs_end).unwrap_or_default();

                for line in block.lines() {
                    let trimmed = super::trim_jsdoc_line(line);
                    if let Some(after_at) = trimmed.strip_prefix('@') {
                        let tag_name = after_at.split_whitespace().next().unwrap_or_default();
                        if EMPTY_TAGS.contains(&tag_name) {
                            let rest = after_at.strip_prefix(tag_name).unwrap_or_default().trim();
                            if !rest.is_empty() {
                                let span_start = u32::try_from(abs_start).unwrap_or(0);
                                let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                                ctx.report_warning(
                                    "jsdoc/empty-tags",
                                    &format!("`@{tag_name}` tag should not have content"),
                                    Span::new(span_start, span_end),
                                );
                            }
                        }
                    }
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(EmptyTags)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
