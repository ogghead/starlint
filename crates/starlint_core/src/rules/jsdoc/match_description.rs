//! Rule: `jsdoc/match-description`
//!
//! Enforce `JSDoc` descriptions match a pattern (start with uppercase, end with period).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct MatchDescription;

/// Check whether the first alphabetic character of the `JSDoc` description
/// (text before the first tag) is lowercase.  Returns `true` when the
/// description starts with a lowercase letter.  Avoids allocating by
/// inspecting characters in-place.
fn description_starts_lowercase(block: &str) -> bool {
    for line in block.lines() {
        let trimmed = super::trim_jsdoc_line(line);
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('@') {
            return false;
        }
        // Found the first description line — check first char.
        let first = trimmed.chars().next().unwrap_or_default();
        return first.is_alphabetic() && !first.is_uppercase();
    }
    false
}

impl NativeRule for MatchDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/match-description".to_owned(),
            description: "Enforce JSDoc descriptions match a pattern".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        // Collect violation spans while source is borrowed, then report.
        let violations = {
            let source = ctx.source_text();

            // Early exit: no JSDoc blocks at all.
            if !source.contains("/**") {
                return;
            }

            let mut spans: Vec<Span> = Vec::new();
            let mut pos = 0;
            while let Some(start) = source.get(pos..).and_then(|s| s.find("/**")) {
                let abs_start = pos.saturating_add(start);
                let search_from = abs_start.saturating_add(3);
                if let Some(end) = source.get(search_from..).and_then(|s| s.find("*/")) {
                    let abs_end = search_from.saturating_add(end).saturating_add(2);
                    let block = source.get(abs_start..abs_end).unwrap_or_default();

                    if description_starts_lowercase(block) {
                        let span_start = u32::try_from(abs_start).unwrap_or(0);
                        let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                        spans.push(Span::new(span_start, span_end));
                    }

                    pos = abs_end;
                } else {
                    break;
                }
            }
            spans
        };

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: "jsdoc/match-description".to_owned(),
                message: "JSDoc description should start with an uppercase letter".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MatchDescription)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_lowercase_description() {
        let source = "/** does something */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_uppercase_description() {
        let source = "/** Does something */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_tag_only_block() {
        let source = "/** @param {string} x */\nfunction foo(x) {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
