//! Rule: `jest/prefer-jest-mocked`
//!
//! Suggest using `jest.mocked()` over manual type casting of mocked functions.
//! Flags `as jest.Mock`, `as jest.MockedFunction`, and similar type assertions.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/prefer-jest-mocked";

/// Type assertion patterns that indicate manual mock typing.
const MOCK_TYPE_PATTERNS: &[&str] = &[
    "as jest.MockedFunction",
    "as jest.MockedClass",
    "as jest.Mocked<",
    "as jest.Mock",
    "<jest.MockedFunction",
    "<jest.MockedClass",
    "<jest.Mock>",
];

/// Suggests using `jest.mocked()` over manual type casting.
#[derive(Debug)]
pub struct PreferJestMocked;

impl NativeRule for PreferJestMocked {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `jest.mocked()` over manual type casting".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            let mut positions: Vec<(usize, usize)> = Vec::new();

            // Check longer patterns first so shorter substrings don't double-match
            for pattern in MOCK_TYPE_PATTERNS {
                let mut search_start: usize = 0;

                while let Some(pos) = source.get(search_start..).and_then(|s| s.find(pattern)) {
                    let abs_pos = search_start.saturating_add(pos);
                    let end_pos = abs_pos.saturating_add(pattern.len());

                    // Only add if this position doesn't overlap an existing match
                    let overlaps = positions.iter().any(|&(s, e)| abs_pos < e && end_pos > s);

                    if !overlaps {
                        positions.push((abs_pos, end_pos));
                    }

                    search_start = end_pos;
                }
            }

            positions
                .into_iter()
                .map(|(start, end)| {
                    let start_u32 = u32::try_from(start).unwrap_or(0);
                    let end_u32 = u32::try_from(end).unwrap_or(start_u32);
                    Span::new(start_u32, end_u32)
                })
                .collect::<Vec<_>>()
        };

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Use `jest.mocked()` instead of manual type casting for mocked functions"
                    .to_owned(),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferJestMocked)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_as_jest_mock() {
        let source = "const mockFn = myFn as jest.Mock;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`as jest.Mock` type cast should be flagged");
    }

    #[test]
    fn test_flags_as_jest_mocked_function() {
        let source = "const mockFn = myFn as jest.MockedFunction<typeof myFn>;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`as jest.MockedFunction` type cast should be flagged"
        );
    }

    #[test]
    fn test_allows_jest_mocked_call() {
        let source = "const mockFn = jest.mocked(myFn);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`jest.mocked()` call should not be flagged"
        );
    }
}
