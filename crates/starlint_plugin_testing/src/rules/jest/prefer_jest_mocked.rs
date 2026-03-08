//! Rule: `jest/prefer-jest-mocked`
//!
//! Suggest using `jest.mocked()` over manual type casting of mocked functions.
//! Flags `as jest.Mock`, `as jest.MockedFunction`, and similar type assertions.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

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

impl LintRule for PreferJestMocked {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `jest.mocked()` over manual type casting".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("jest.Mock")
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
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
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferJestMocked)];
        lint_source(source, "test.js", &rules)
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
