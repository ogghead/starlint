//! Rule: `typescript/prefer-includes`
//!
//! Prefer `.includes()` over `.indexOf() !== -1` and similar patterns.
//! Using `.includes()` is more readable and expressive than checking the
//! result of `.indexOf()` against `-1` or `0`.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! Flagged patterns:
//! - `.indexOf(x) !== -1`
//! - `.indexOf(x) != -1`
//! - `.indexOf(x) >= 0`
//! - `.indexOf(x) > -1`
//! - `.indexOf(x) === -1` (negated check)
//! - `.indexOf(x) == -1` (negated check)
//! - `.indexOf(x) < 0`

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.indexOf()` comparisons that can be replaced with `.includes()`.
#[derive(Debug)]
pub struct PreferIncludes;

impl NativeRule for PreferIncludes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-includes".to_owned(),
            description: "Prefer `.includes()` over `.indexOf()` comparison".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_indexof_comparisons(source);

        for (start, end) in findings {
            ctx.report_warning(
                "typescript/prefer-includes",
                "Use `.includes()` instead of `.indexOf()` comparison",
                Span::new(start, end),
            );
        }
    }
}

/// Comparison patterns that follow `.indexOf(...)`.
const INDEXOF_COMPARISONS: &[&str] = &["!== -1", "!= -1", ">= 0", "> -1", "=== -1", "== -1", "< 0"];

/// Scan source text for `.indexOf(` patterns followed by comparison operators.
///
/// Returns a list of `(start_offset, end_offset)` for each occurrence.
fn find_indexof_comparisons(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let needle = ".indexOf(";
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(needle)) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_needle = absolute_pos.saturating_add(needle.len());

        // Find the closing parenthesis for indexOf(...)
        if let Some(close_paren) = find_matching_paren(source, after_needle) {
            let after_close = close_paren.saturating_add(1);
            let rest = source.get(after_close..).unwrap_or("");
            let trimmed = rest.trim_start();
            let whitespace_len = rest.len().saturating_sub(trimmed.len());

            // Check if the rest starts with a known comparison pattern
            for pattern in INDEXOF_COMPARISONS {
                if trimmed.starts_with(pattern) {
                    let pattern_end = after_close
                        .saturating_add(whitespace_len)
                        .saturating_add(pattern.len());
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferIncludes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_indexof_not_equal_minus_one() {
        let diags = lint("if (arr.indexOf(x) !== -1) {}");
        assert_eq!(diags.len(), 1, ".indexOf() !== -1 should be flagged");
    }

    #[test]
    fn test_flags_indexof_gte_zero() {
        let diags = lint("if (arr.indexOf(x) >= 0) {}");
        assert_eq!(diags.len(), 1, ".indexOf() >= 0 should be flagged");
    }

    #[test]
    fn test_flags_indexof_equal_minus_one() {
        let diags = lint("if (arr.indexOf(x) === -1) {}");
        assert_eq!(diags.len(), 1, ".indexOf() === -1 should be flagged");
    }

    #[test]
    fn test_flags_indexof_gt_minus_one() {
        let diags = lint("if (str.indexOf('a') > -1) {}");
        assert_eq!(diags.len(), 1, ".indexOf() > -1 should be flagged");
    }

    #[test]
    fn test_allows_indexof_other_usage() {
        let diags = lint("const idx = arr.indexOf(x);");
        assert!(
            diags.is_empty(),
            ".indexOf() used without comparison should not be flagged"
        );
    }
}
