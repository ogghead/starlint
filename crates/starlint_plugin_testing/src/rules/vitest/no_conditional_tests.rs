//! Rule: `vitest/no-conditional-tests`
//!
//! Warn when `if` or `switch` statements are used inside test callbacks.
//! Conditional logic in tests makes them harder to understand and can hide
//! untested code paths. Extract each branch into a separate test instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/no-conditional-tests";

/// Test function names that define test bodies.
const TEST_FN_NAMES: &[&str] = &["it", "test"];

/// Warn when `if`/`switch` is used inside a test callback.
#[derive(Debug)]
pub struct NoConditionalTests;

impl LintRule for NoConditionalTests {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow conditional logic inside test callbacks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("it(")
            || source_text.contains("test(")
            || source_text.contains("describe("))
            && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();

        // Scan for `if (` or `switch (` that appear inside test/it callbacks.
        // Heuristic: find test/it calls, then check if any if/switch keywords
        // appear between the opening and closing braces of the callback body.
        let violations = find_conditionals_in_tests(source);

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Avoid conditional logic (`if`/`switch`) inside tests — split into separate test cases instead".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Find `if`/`switch` statements inside test/it callback bodies.
///
/// Uses a text-scanning approach: locate `it(`/`test(` calls, find the
/// callback body, then check for `if ` or `switch ` keywords within it.
fn find_conditionals_in_tests(source: &str) -> Vec<Span> {
    let mut results = Vec::new();

    for test_fn in TEST_FN_NAMES {
        let pattern = format!("{test_fn}(");
        let mut search_start: usize = 0;

        while let Some(pos) = source.get(search_start..).and_then(|s| s.find(&pattern)) {
            let abs_pos = search_start.saturating_add(pos);
            // Find the opening brace of the callback body after the test call.
            let after_call = abs_pos.saturating_add(pattern.len());

            if let Some(body_region) = source.get(after_call..) {
                // Find a `{` that starts the callback body (skip the test description string).
                if let Some(brace_offset) = body_region.find('{') {
                    let body_start = after_call.saturating_add(brace_offset);

                    // Find the matching closing brace (simple brace counting).
                    if let Some(body_end) = find_matching_brace(source, body_start) {
                        let body = source.get(body_start..body_end).unwrap_or("");

                        // Check for `if (` or `switch (` inside the body.
                        check_for_conditional(body, body_start, "if ", &mut results);
                        check_for_conditional(body, body_start, "switch ", &mut results);
                    }
                }
            }

            search_start = abs_pos.saturating_add(1);
        }
    }

    results
}

/// Check if a conditional keyword appears in the body and record spans.
fn check_for_conditional(body: &str, body_offset: usize, keyword: &str, results: &mut Vec<Span>) {
    let mut search: usize = 0;
    while let Some(pos) = body.get(search..).and_then(|s| s.find(keyword)) {
        let abs = body_offset.saturating_add(search).saturating_add(pos);
        let kw_len = keyword.trim_end().len();
        let start = u32::try_from(abs).unwrap_or(0);
        let end = u32::try_from(abs.saturating_add(kw_len)).unwrap_or(start);
        results.push(Span::new(start, end));
        search = search.saturating_add(pos).saturating_add(1);
    }
}

/// Find the matching closing brace for the brace at `open_pos`.
fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    let mut depth: usize = 0;
    for (i, ch) in source.get(open_pos..)?.char_indices() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_pos.saturating_add(i).saturating_add(1));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoConditionalTests);

    #[test]
    fn test_flags_if_inside_test() {
        let source = r#"test("my test", () => { if (true) { doSomething(); } });"#;
        let diags = lint(source);
        assert!(
            !diags.is_empty(),
            "`if` inside a test callback should be flagged"
        );
    }

    #[test]
    fn test_flags_switch_inside_it() {
        let source = r#"it("my test", () => { switch (x) { case 1: break; } });"#;
        let diags = lint(source);
        assert!(
            !diags.is_empty(),
            "`switch` inside an `it` callback should be flagged"
        );
    }

    #[test]
    fn test_allows_test_without_conditionals() {
        let source = r#"test("my test", () => { expect(1).toBe(1); });"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "test without conditionals should not be flagged"
        );
    }
}
