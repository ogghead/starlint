//! Rule: `jsdoc/check-access`
//!
//! Enforce valid `@access` tags in `JSDoc` comments.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct CheckAccess;

impl NativeRule for CheckAccess {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/check-access".to_owned(),
            description: "Enforce valid `@access` tags in JSDoc comments".to_owned(),
            category: Category::Correctness,
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
                    if trimmed.starts_with("@access") {
                        let value = trimmed.strip_prefix("@access").unwrap_or_default().trim();
                        if !matches!(value, "public" | "private" | "protected" | "package")
                            && !value.is_empty()
                        {
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                            ctx.report_warning(
                                "jsdoc/check-access",
                                &format!(
                                    "Invalid `@access` value: `{value}`. Use public, private, protected, or package"
                                ),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CheckAccess)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_invalid_access() {
        let source = "/** @access foobar */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_access() {
        let source = "/** @access private */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_all_valid_values() {
        for val in &["public", "private", "protected", "package"] {
            let source = format!("/** @access {val} */\nfunction foo() {{}}");
            let diags = lint(&source);
            assert!(diags.is_empty(), "should allow @access {val}");
        }
    }
}
