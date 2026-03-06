//! Rule: `jsdoc/require-description`
//!
//! Require `JSDoc` comments have a non-empty description.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct RequireDescription;

/// Check if a `JSDoc` block has a description (text before the first tag).
fn has_description(block: &str) -> bool {
    for line in block.lines() {
        let trimmed = super::trim_jsdoc_line(line);
        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }
        // If we hit a tag first, no description
        if trimmed.starts_with('@') {
            return false;
        }
        // Non-empty, non-tag content is a description
        return true;
    }
    false
}

impl NativeRule for RequireDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/require-description".to_owned(),
            description: "Require JSDoc comments have a description".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
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

                if !has_description(block) {
                    let span_start = u32::try_from(abs_start).unwrap_or(0);
                    let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                    ctx.report(Diagnostic {
                        rule_name: "jsdoc/require-description".to_owned(),
                        message: "JSDoc comment is missing a description".to_owned(),
                        span: Span::new(span_start, span_end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireDescription)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_missing_description() {
        let source = "/** @param {string} x */\nfunction foo(x) {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_with_description() {
        let source = "/** Does something.\n * @param {string} x\n */\nfunction foo(x) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_inline_description() {
        let source = "/** Does something */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
