//! Rule: `jsdoc/no-restricted-syntax`
//!
//! Forbid certain `JSDoc` tags (configurable, defaults to forbidding `@todo`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default restricted tags.
const DEFAULT_RESTRICTED: &[&str] = &["todo"];

#[derive(Debug)]
pub struct NoRestrictedSyntax;

impl NativeRule for NoRestrictedSyntax {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/no-restricted-syntax".to_owned(),
            description: "Forbid certain JSDoc tags".to_owned(),
            category: Category::Suggestion,
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

                for line in block.lines() {
                    let trimmed = super::trim_jsdoc_line(line);
                    if let Some(after_at) = trimmed.strip_prefix('@') {
                        let tag_name = after_at.split_whitespace().next().unwrap_or_default();
                        if DEFAULT_RESTRICTED.contains(&tag_name) {
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                            ctx.report(Diagnostic {
                                rule_name: "jsdoc/no-restricted-syntax".to_owned(),
                                message: format!("Restricted JSDoc tag: `@{tag_name}`"),
                                span: Span::new(span_start, span_end),
                                severity: Severity::Warning,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestrictedSyntax)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_restricted_tag() {
        let source = "/** @todo fix this */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_unrestricted_tags() {
        let source = "/** @param {string} x */\nfunction foo(x) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_tags() {
        let source = "/** Just a description */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
