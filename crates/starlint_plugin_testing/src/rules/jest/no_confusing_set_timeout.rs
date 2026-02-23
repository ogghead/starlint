//! Rule: `jest/no-confusing-set-timeout`
//!
//! Warn when `jest.setTimeout()` is used inside test blocks instead of at the
//! describe level or top level.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-confusing-set-timeout";

/// Test block function names.
const TEST_BLOCK_NAMES: &[&str] = &[
    "it",
    "test",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
];

/// Flags `jest.setTimeout()` calls that appear inside test/hook blocks.
#[derive(Debug)]
pub struct NoConfusingSetTimeout;

impl LintRule for NoConfusingSetTimeout {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `jest.setTimeout` inside test blocks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("setTimeout") && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            let needle = "jest.setTimeout(";
            let mut violations: Vec<Span> = Vec::new();
            let mut search_start: usize = 0;

            while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
                let abs_pos = search_start.saturating_add(pos);

                if is_inside_test_block(source, abs_pos) {
                    let end = abs_pos.saturating_add(needle.len());
                    let span_end = source
                        .get(end..)
                        .and_then(|s| s.find(')'))
                        .map_or(end, |p| end.saturating_add(p).saturating_add(1));

                    let start_u32 = u32::try_from(abs_pos).unwrap_or(0);
                    let end_u32 = u32::try_from(span_end).unwrap_or(start_u32);
                    violations.push(Span::new(start_u32, end_u32));
                }

                search_start = abs_pos.saturating_add(needle.len());
            }

            violations
        };

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`jest.setTimeout()` should be called at the top level or in a `describe` block, not inside a test or hook".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Heuristic: check if a position is inside a test/hook block by scanning
/// backwards for `it(`, `test(`, `beforeEach(`, etc.
fn is_inside_test_block(source: &str, pos: usize) -> bool {
    let before = source.get(..pos).unwrap_or("");

    // Count open/close braces to determine nesting.
    // If we find a test block name followed by `(` at a nesting level that
    // encompasses our position, we're inside a test block.
    for test_name in TEST_BLOCK_NAMES {
        let pattern = format!("{test_name}(");
        let mut search_from: usize = 0;

        while let Some(name_pos) = before.get(search_from..).and_then(|s| s.find(&pattern)) {
            let abs_name_pos = search_from.saturating_add(name_pos);
            let after_name = abs_name_pos.saturating_add(pattern.len());

            // Find the opening brace of the callback body after this test call
            if let Some(brace_offset) = source.get(after_name..pos).and_then(|s| s.find('{')) {
                let brace_pos = after_name.saturating_add(brace_offset);
                let between = source.get(brace_pos..pos).unwrap_or("");

                // Count braces to see if we are still inside this block
                let open_count = between.chars().filter(|c| *c == '{').count();
                let close_count = between.chars().filter(|c| *c == '}').count();

                if open_count > close_count {
                    return true;
                }
            }

            search_from = abs_name_pos.saturating_add(1);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConfusingSetTimeout)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_set_timeout_in_test() {
        let source = r"
test('foo', () => {
    jest.setTimeout(10000);
    expect(1).toBe(1);
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`jest.setTimeout` inside test block should be flagged"
        );
    }

    #[test]
    fn test_allows_set_timeout_at_top_level() {
        let source = r"
jest.setTimeout(10000);
test('foo', () => {
    expect(1).toBe(1);
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`jest.setTimeout` at top level should not be flagged"
        );
    }

    #[test]
    fn test_flags_set_timeout_in_before_each() {
        let source = r"
beforeEach(() => {
    jest.setTimeout(5000);
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`jest.setTimeout` inside `beforeEach` should be flagged"
        );
    }
}
