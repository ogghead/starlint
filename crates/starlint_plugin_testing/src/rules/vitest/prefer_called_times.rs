//! Rule: `vitest/prefer-called-times`
//!
//! Suggest `toHaveBeenCalledTimes(n)` over manual `.mock.calls.length` checks.
//! Using the built-in matcher is more expressive and produces better error
//! messages when assertions fail.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-called-times";

/// Suggest `toHaveBeenCalledTimes(n)` over `.mock.calls.length` checks.
#[derive(Debug)]
pub struct PreferCalledTimes;

impl LintRule for PreferCalledTimes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toHaveBeenCalledTimes()` over manual `.mock.calls.length` checks"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = find_mock_calls_length(ctx.source_text());

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Prefer `toHaveBeenCalledTimes(n)` over checking `.mock.calls.length` manually"
                        .to_owned(),
                span,
                severity: Severity::Warning,
                help: Some("Use `toHaveBeenCalledTimes()` matcher instead".to_owned()),
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Find `.mock.calls.length` patterns in source text.
fn find_mock_calls_length(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let needle = ".mock.calls.length";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(pos);
        let start = u32::try_from(abs_pos).unwrap_or(0);
        let end = u32::try_from(abs_pos.saturating_add(needle.len())).unwrap_or(start);
        results.push(Span::new(start, end));
        search_start = abs_pos.saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferCalledTimes)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_mock_calls_length() {
        let source = "expect(fn.mock.calls.length).toBe(2);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`.mock.calls.length` check should be flagged"
        );
    }

    #[test]
    fn test_allows_to_have_been_called_times() {
        let source = "expect(fn).toHaveBeenCalledTimes(2);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledTimes` should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_length_access() {
        let source = "expect(arr.length).toBe(3);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "unrelated `.length` access should not be flagged"
        );
    }
}
