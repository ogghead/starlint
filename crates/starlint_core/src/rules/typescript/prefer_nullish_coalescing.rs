//! Rule: `typescript/prefer-nullish-coalescing`
//!
//! Prefer `??` (nullish coalescing) over `||` (logical OR) for default values
//! when the left side may be nullish but not falsy. Using `||` treats `0`, `""`,
//! `false`, and `NaN` as falsy, which can lead to unexpected behavior when the
//! intent is only to handle `null` or `undefined`.
//!
//! Since full type analysis is unavailable, this rule uses a simplified
//! heuristic: it flags `||` when preceded by an optional chain expression
//! (e.g. `foo?.bar || default`), because optional chaining already signals
//! null-awareness and `??` is almost always the correct operator in that context.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `?.` followed by `||` patterns where `??` is likely more appropriate.
#[derive(Debug)]
pub struct PreferNullishCoalescing;

impl NativeRule for PreferNullishCoalescing {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-nullish-coalescing".to_owned(),
            description: "Prefer `??` over `||` when the left side uses optional chaining"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings = find_optional_chain_or_patterns(ctx.source_text());

        for (start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/prefer-nullish-coalescing".to_owned(),
                message: "Prefer `??` over `||` after optional chaining — `||` also catches falsy values like `0`, `\"\"`, and `false`".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Replace `||` with `??`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace `||` with `??`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(start, end),
                        replacement: "??".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Scan source text for `?.` ... `||` patterns on the same line.
///
/// The heuristic works line-by-line: if a line contains `?.` followed by `||`,
/// the `||` is flagged because optional chaining implies null-awareness, and
/// `??` is almost always the intended operator.
///
/// Returns a list of `(start_offset, end_offset)` tuples pointing at the `||`.
fn find_optional_chain_or_patterns(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let mut line_start: usize = 0;

    for line in source.split('\n') {
        // Find `?.` in this line
        if let Some(opt_chain_pos) = line.find("?.") {
            // Search for `||` after the `?.`
            let search_start = opt_chain_pos.saturating_add(2);
            if let Some(or_offset) = line.get(search_start..).and_then(|s| s.find("||")) {
                let absolute_or = line_start
                    .saturating_add(search_start)
                    .saturating_add(or_offset);

                // Make sure it's `||` and not `||=`
                let after_or = absolute_or.saturating_add(2);
                let is_logical_or_assign = source
                    .get(after_or..after_or.saturating_add(1))
                    .is_none_or(|ch| ch == "=");

                if !is_logical_or_assign
                    || source
                        .get(after_or..after_or.saturating_add(1))
                        .is_none_or(|ch| ch != "=")
                {
                    // Re-check: only flag plain `||`, not `||=`
                    let next_char = source
                        .get(after_or..after_or.saturating_add(1))
                        .unwrap_or("");
                    if next_char != "=" {
                        let start = u32::try_from(absolute_or).unwrap_or(0);
                        let end = u32::try_from(after_or).unwrap_or(start);
                        results.push((start, end));
                    }
                }
            }
        }

        line_start = line_start.saturating_add(line.len()).saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferNullishCoalescing)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_optional_chain_with_or() {
        let diags = lint("const x = foo?.bar || 'default';");
        assert_eq!(diags.len(), 1, "`foo?.bar || default` should be flagged");
    }

    #[test]
    fn test_flags_deep_optional_chain_with_or() {
        let diags = lint("const x = a?.b?.c || fallback;");
        assert_eq!(
            diags.len(),
            1,
            "deep optional chain with `||` should be flagged"
        );
    }

    #[test]
    fn test_allows_optional_chain_with_nullish_coalescing() {
        let diags = lint("const x = foo?.bar ?? 'default';");
        assert!(
            diags.is_empty(),
            "`??` after optional chain should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_or_without_optional_chain() {
        let diags = lint("const x = foo || 'default';");
        assert!(
            diags.is_empty(),
            "`||` without optional chain should not be flagged"
        );
    }

    #[test]
    fn test_allows_logical_or_assignment() {
        let diags = lint("foo?.bar ||= 'default';");
        assert!(
            diags.is_empty(),
            "`||=` should not be flagged even after optional chain"
        );
    }
}
