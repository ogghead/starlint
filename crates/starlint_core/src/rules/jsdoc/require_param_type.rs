//! Rule: `jsdoc/require-param-type`
//!
//! Require `@param` tags have type annotations.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct RequireParamType;

impl NativeRule for RequireParamType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/require-param-type".to_owned(),
            description: "Require `@param` tags have type annotations".to_owned(),
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
                    if let Some(tag_rest) = trimmed.strip_prefix("@param") {
                        let tag_content = tag_rest.trim();
                        if !tag_content.starts_with('{') {
                            // No type annotation
                            let param_name = tag_content
                                .split_whitespace()
                                .next()
                                .unwrap_or_default()
                                .trim_start_matches('[')
                                .split('=')
                                .next()
                                .unwrap_or_default()
                                .trim_end_matches(']');
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                            ctx.report_warning(
                                "jsdoc/require-param-type",
                                &format!("`@param {param_name}` is missing a type annotation"),
                                Span::new(span_start, span_end),
                            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireParamType)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_param_without_type() {
        let source = "/** @param x The value */\nfunction foo(x) {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_param_with_type() {
        let source = "/** @param {string} x The value */\nfunction foo(x) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_multiple_untyped_params() {
        let source = "/** @param x\n * @param y */\nfunction foo(x, y) {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 2);
    }
}
