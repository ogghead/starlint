//! Rule: `jsdoc/check-tag-names`
//!
//! Enforce valid `JSDoc` tag names.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Known valid `JSDoc` tags.
const VALID_TAGS: &[&str] = &[
    "abstract",
    "access",
    "alias",
    "async",
    "augments",
    "author",
    "borrows",
    "callback",
    "class",
    "classdesc",
    "constant",
    "constructs",
    "copyright",
    "default",
    "defaultvalue",
    "deprecated",
    "description",
    "enum",
    "event",
    "example",
    "exports",
    "extends",
    "external",
    "file",
    "fileoverview",
    "fires",
    "function",
    "generator",
    "global",
    "hideconstructor",
    "ignore",
    "implements",
    "import",
    "inheritdoc",
    "inner",
    "instance",
    "interface",
    "kind",
    "lends",
    "license",
    "link",
    "listens",
    "member",
    "memberof",
    "method",
    "mixes",
    "mixin",
    "module",
    "name",
    "namespace",
    "override",
    "package",
    "param",
    "private",
    "prop",
    "property",
    "protected",
    "public",
    "readonly",
    "requires",
    "return",
    "returns",
    "see",
    "since",
    "static",
    "summary",
    "template",
    "this",
    "throws",
    "todo",
    "tutorial",
    "type",
    "typedef",
    "var",
    "variation",
    "version",
    "virtual",
    "yields",
];

#[derive(Debug)]
pub struct CheckTagNames;

impl NativeRule for CheckTagNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/check-tag-names".to_owned(),
            description: "Enforce valid JSDoc tag names".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
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

                let mut line_offset = abs_start;
                for line in block.lines() {
                    let trimmed = super::trim_jsdoc_line(line);
                    if let Some(after_at) = trimmed.strip_prefix('@') {
                        let tag_name = after_at
                            .split_whitespace()
                            .next()
                            .unwrap_or_default()
                            .trim_end_matches('{');
                        if !tag_name.is_empty() && !VALID_TAGS.contains(&tag_name) {
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);

                            // Try to find a valid tag that matches case-insensitively
                            let lower = tag_name.to_ascii_lowercase();
                            let suggestion = VALID_TAGS
                                .iter()
                                .find(|t| t.to_ascii_lowercase() == lower)
                                .copied();

                            // Build a fix if we have a suggestion and can locate it
                            let fix = suggestion.and_then(|correct_tag| {
                                let tag_with_at = format!("@{tag_name}");
                                let line_source = source.get(line_offset..).unwrap_or_default();
                                line_source.find(&tag_with_at).map(|at_pos| {
                                    // Span covers just the tag name (after `@`)
                                    let tag_abs =
                                        line_offset.saturating_add(at_pos).saturating_add(1); // skip `@`
                                    let tag_start = u32::try_from(tag_abs).unwrap_or(0);
                                    let tag_end = tag_start
                                        .saturating_add(u32::try_from(tag_name.len()).unwrap_or(0));
                                    Fix {
                                        message: format!(
                                            "Replace `@{tag_name}` with `@{correct_tag}`"
                                        ),
                                        edits: vec![Edit {
                                            span: Span::new(tag_start, tag_end),
                                            replacement: correct_tag.to_owned(),
                                        }],
                                    }
                                })
                            });

                            let help = suggestion
                                .map(|correct_tag| format!("Did you mean `@{correct_tag}`?"));

                            ctx.report(Diagnostic {
                                rule_name: "jsdoc/check-tag-names".to_owned(),
                                message: format!("Unknown JSDoc tag: `@{tag_name}`"),
                                span: Span::new(span_start, span_end),
                                severity: Severity::Warning,
                                help,
                                fix,
                                labels: vec![],
                            });
                        }
                    }
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CheckTagNames)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_unknown_tag() {
        let source = "/** @foobar */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_known_tags() {
        let source = "/** @param {string} name */\nfunction foo(name) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_returns_tag() {
        let source = "/** @returns {number} The result */\nfunction foo() { return 1; }";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
