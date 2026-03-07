//! Rule: `typescript/no-misused-spread`
//!
//! Disallow spreading non-iterable values. Spreading a non-iterable value
//! like an object literal inside an array literal (`[...{}]`) is almost
//! certainly a mistake. Object literals are not iterable and will cause a
//! runtime `TypeError`.
//!
//! This rule uses a simplified text-based approach via `run_once()`: it scans
//! the source for `[...{` patterns which indicate spreading an object literal
//! into an array context.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `[...{` patterns where a non-iterable object is spread into an array.
#[derive(Debug)]
pub struct NoMisusedSpread;

impl LintRule for NoMisusedSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-misused-spread".to_owned(),
            description: "Disallow spreading non-iterable values into arrays".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_misused_spreads(source);

        for (start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-misused-spread".to_owned(),
                message: "Do not spread a non-iterable value into an array — object literals are not iterable".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// The pattern to detect: `[...{` with optional whitespace between `...` and `{`.
const SPREAD_PATTERN: &str = "[...";

/// Scan source text for `[...{` patterns indicating an object literal spread
/// into an array context.
///
/// Returns a list of `(start_offset, end_offset)` tuples for each occurrence.
fn find_misused_spreads(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source
        .get(search_from..)
        .and_then(|s| s.find(SPREAD_PATTERN))
    {
        let absolute_pos = search_from.saturating_add(pos);
        let after_spread = absolute_pos.saturating_add(SPREAD_PATTERN.len());

        // Check if the character after `[...` (skipping whitespace) is `{`,
        // which means an object literal is being spread into an array.
        let rest = source.get(after_spread..).unwrap_or("");
        let trimmed = rest.trim_start();

        if trimmed.starts_with('{') {
            // Find the end of the pattern for the span — use a reasonable
            // length covering `[...{`.
            let pattern_end = after_spread
                .saturating_add(rest.len().saturating_sub(trimmed.len()).saturating_add(1));
            let start = u32::try_from(absolute_pos).unwrap_or(0);
            let end = u32::try_from(pattern_end).unwrap_or(start);
            results.push((start, end));
        }

        search_from = after_spread;
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMisusedSpread)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_object_spread_in_array() {
        let diags = lint("const arr = [...{a: 1}];");
        assert_eq!(
            diags.len(),
            1,
            "spreading an object literal into an array should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_object_spread_in_array() {
        let diags = lint("const arr = [...{}];");
        assert_eq!(
            diags.len(),
            1,
            "spreading an empty object into an array should be flagged"
        );
    }

    #[test]
    fn test_flags_spread_with_whitespace() {
        let diags = lint("const arr = [... { a: 1 }];");
        assert_eq!(
            diags.len(),
            1,
            "spreading an object with whitespace before brace should be flagged"
        );
    }

    #[test]
    fn test_allows_array_spread() {
        let diags = lint("const arr = [...otherArr];");
        assert!(
            diags.is_empty(),
            "spreading an array into an array should not be flagged"
        );
    }

    #[test]
    fn test_allows_object_spread_in_object() {
        let diags = lint("const obj = {...other};");
        assert!(
            diags.is_empty(),
            "spreading in an object context should not be flagged"
        );
    }
}
