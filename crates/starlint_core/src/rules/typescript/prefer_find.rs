//! Rule: `typescript/prefer-find`
//!
//! Prefer `.find()` over `.filter()[0]` or `.filter().at(0)` patterns.
//! Using `.find()` is more efficient and expressive — it short-circuits
//! on the first match instead of building an entire filtered array.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! Flagged patterns:
//! - `.filter(...)[0]`
//! - `.filter(...).at(0)`
//! - `.filter(...)?.at(0)`

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-find";

/// Flags `.filter(...)[0]` and `.filter(...).at(0)` patterns that should use `.find()`.
#[derive(Debug)]
pub struct PreferFind;

impl NativeRule for PreferFind {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `.find()` over `.filter()[0]` or `.filter().at(0)`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();
        let findings = find_filter_first_access(&source);

        for (start, end) in findings {
            // Build fix: replace .filter(cb)[0] with .find(cb)
            let span_text = source
                .get(start as usize..end as usize)
                .unwrap_or("")
                .to_owned();
            let fix = build_filter_to_find_fix(&span_text, start, end);

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Use `.find()` instead of `.filter()` followed by index access".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Replace `.filter(...)` with `.find(...)`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Build a fix that replaces `.filter(cb)[0]` with `.find(cb)`.
#[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
fn build_filter_to_find_fix(span_text: &str, start: u32, end: u32) -> Option<Fix> {
    // Replace `.filter(` with `.find(` and strip the suffix ([0], .at(0), ?.at(0))
    let mut replacement = span_text.replacen(".filter(", ".find(", 1);
    for suffix in FIRST_ACCESS_PATTERNS {
        if replacement.ends_with(suffix) {
            let new_len = replacement.len().saturating_sub(suffix.len());
            replacement.truncate(new_len);
            return Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace `.filter(...)` with `.find(...)`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(start, end),
                    replacement,
                }],
                is_snippet: false,
            });
        }
    }
    None
}

/// Suffix patterns that indicate first-element access after `.filter(...)`.
const FIRST_ACCESS_PATTERNS: &[&str] = &["[0]", "?.at(0)", ".at(0)"];

/// Scan source text for `.filter(` patterns followed by `)` and then a
/// first-element access (`[0]`, `.at(0)`, `?.at(0)`).
///
/// Returns a list of `(start_offset, end_offset)` for each occurrence.
fn find_filter_first_access(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let needle = ".filter(";
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(needle)) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_needle = absolute_pos.saturating_add(needle.len());

        // Find the closing parenthesis for filter(...)
        if let Some(close_paren) = find_matching_paren(source, after_needle) {
            let after_close = close_paren.saturating_add(1);
            let rest = source.get(after_close..).unwrap_or("");

            // Check if the rest starts with a known first-access pattern
            for pattern in FIRST_ACCESS_PATTERNS {
                if rest.starts_with(pattern) {
                    let pattern_end = after_close.saturating_add(pattern.len());
                    let start = u32::try_from(absolute_pos).unwrap_or(0);
                    let end = u32::try_from(pattern_end).unwrap_or(start);
                    results.push((start, end));
                    break;
                }
            }
        }

        search_from = after_needle;
    }

    results
}

/// Find the matching closing parenthesis, handling nesting.
///
/// `start` is the position right after the opening `(`.
/// Returns the position of the matching `)`.
fn find_matching_paren(source: &str, start: usize) -> Option<usize> {
    let mut depth: u32 = 1;
    let mut pos = start;
    let bytes = source.as_bytes();
    let len = bytes.len();

    while pos < len {
        match bytes.get(pos).copied() {
            Some(b'(') => depth = depth.saturating_add(1),
            Some(b')') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
        pos = pos.saturating_add(1);
    }

    None
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferFind)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_filter_bracket_zero() {
        let diags = lint("const first = arr.filter(x => x > 0)[0];");
        assert_eq!(diags.len(), 1, ".filter(...)[0] should be flagged");
    }

    #[test]
    fn test_flags_filter_dot_at_zero() {
        let diags = lint("const first = arr.filter(x => x > 0).at(0);");
        assert_eq!(diags.len(), 1, ".filter(...).at(0) should be flagged");
    }

    #[test]
    fn test_flags_filter_optional_at_zero() {
        let diags = lint("const first = arr.filter(x => x > 0)?.at(0);");
        assert_eq!(diags.len(), 1, ".filter(...)?.at(0) should be flagged");
    }

    #[test]
    fn test_allows_filter_alone() {
        let diags = lint("const filtered = arr.filter(x => x > 0);");
        assert!(
            diags.is_empty(),
            ".filter() without index access should not be flagged"
        );
    }

    #[test]
    fn test_allows_find_usage() {
        let diags = lint("const first = arr.find(x => x > 0);");
        assert!(diags.is_empty(), ".find() should not be flagged");
    }
}
