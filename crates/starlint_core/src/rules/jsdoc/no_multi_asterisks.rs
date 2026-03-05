//! Rule: `jsdoc/no-multi-asterisks`
//!
//! Forbid multiple asterisks at the start of `JSDoc` lines.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct NoMultiAsterisks;

impl NativeRule for NoMultiAsterisks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/no-multi-asterisks".to_owned(),
            description: "Forbid multiple asterisks in JSDoc comments".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
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

                // Check inner lines (skip first line with /**, and last with */)
                let lines: Vec<&str> = block.lines().collect();
                let mut edits = Vec::new();
                let mut line_offset = abs_start;
                for (i, line) in lines.iter().enumerate() {
                    if i == 0 || i == lines.len().saturating_sub(1) {
                        line_offset = line_offset.saturating_add(line.len()).saturating_add(1);
                        continue;
                    }
                    let trimmed = line.trim();
                    // Count leading asterisks
                    let asterisk_count = trimmed.chars().take_while(|c| *c == '*').count();
                    if asterisk_count > 1 {
                        // Find the position of the asterisks in the original source line
                        let whitespace_prefix = line.len().saturating_sub(trimmed.len());
                        // The first `*` stays; remove the extra ones
                        let extra_start = line_offset
                            .saturating_add(whitespace_prefix)
                            .saturating_add(1); // skip the first `*`
                        let extra_end =
                            extra_start.saturating_add(asterisk_count.saturating_sub(1));
                        edits.push(Edit {
                            span: Span::new(
                                u32::try_from(extra_start).unwrap_or(0),
                                u32::try_from(extra_end).unwrap_or(0),
                            ),
                            replacement: String::new(),
                        });
                    }
                    line_offset = line_offset.saturating_add(line.len()).saturating_add(1);
                }

                if !edits.is_empty() {
                    let span_start = u32::try_from(abs_start).unwrap_or(0);
                    let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                    ctx.report(Diagnostic {
                        rule_name: "jsdoc/no-multi-asterisks".to_owned(),
                        message: "Multiple asterisks at the start of a JSDoc line".to_owned(),
                        span: Span::new(span_start, span_end),
                        severity: Severity::Warning,
                        help: Some("Replace multiple asterisks with a single one".to_owned()),
                        fix: Some(Fix {
                            message: "Remove extra asterisks".to_owned(),
                            edits,
                        }),
                        labels: vec![],
                    });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMultiAsterisks)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_multi_asterisks() {
        let source = "/**\n ** Bad line\n */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_single_asterisk() {
        let source = "/**\n * Good line\n */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_inline_comment() {
        let source = "/** Single line comment */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
