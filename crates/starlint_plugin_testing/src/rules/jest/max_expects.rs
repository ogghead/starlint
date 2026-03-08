//! Rule: `jest/max-expects`
//!
//! Warn when a test has too many `expect()` calls (default: > 5).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/max-expects";

/// Default maximum number of `expect()` calls per test.
const DEFAULT_MAX: usize = 5;

/// Tracks test nesting and `expect()` counts.
#[derive(Debug)]
pub struct MaxExpects {
    /// Stack of expect counts per test scope depth.
    /// We use source-text heuristic scanning since the traversal is flat.
    max: usize,
}

impl Default for MaxExpects {
    fn default() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl MaxExpects {
    /// Create a new rule with the default max.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl LintRule for MaxExpects {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Limit the number of `expect()` calls per test".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = {
            let source = ctx.source_text();

            // Early exit: skip files without test calls.
            if !source.contains("it(") && !source.contains("test(") {
                return;
            }

            let test_names = ["it(", "test("];
            let mut violations: Vec<(usize, Span)> = Vec::new();

            for test_name in &test_names {
                let mut search_start: usize = 0;

                while let Some(pos) = source.get(search_start..).and_then(|s| s.find(test_name)) {
                    let abs_pos = search_start.saturating_add(pos);

                    let is_word_boundary = abs_pos == 0
                        || source
                            .as_bytes()
                            .get(abs_pos.saturating_sub(1))
                            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

                    if is_word_boundary {
                        if let Some(body) = extract_test_body(source, abs_pos) {
                            let expect_count = count_expect_calls(body);
                            if expect_count > self.max {
                                let end = abs_pos
                                    .saturating_add(test_name.len())
                                    .saturating_add(body.len());
                                let start_u32 = u32::try_from(abs_pos).unwrap_or(0);
                                let end_u32 = u32::try_from(end).unwrap_or(start_u32);
                                violations.push((expect_count, Span::new(start_u32, end_u32)));
                            }
                        }
                    }

                    search_start = abs_pos.saturating_add(test_name.len());
                }
            }

            violations
        };

        for (expect_count, span) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Test has {expect_count} `expect()` calls (max: {})",
                    self.max,
                ),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Extract the body of a test callback as a string slice.
fn extract_test_body(source: &str, test_start: usize) -> Option<&str> {
    let rest = source.get(test_start..)?;

    // Find the opening brace of the callback
    let brace_pos = rest.find('{')?;
    let body_start = brace_pos.saturating_add(1);

    let body_source = rest.get(body_start..)?;
    let mut depth: usize = 1;

    for (i, ch) in body_source.char_indices() {
        if ch == '{' {
            depth = depth.saturating_add(1);
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return body_source.get(..i);
            }
        }
    }

    None
}

/// Count `expect(` calls in a string, ensuring word boundary.
fn count_expect_calls(body: &str) -> usize {
    let needle = "expect(";
    let mut count: usize = 0;
    let mut search_from: usize = 0;

    while let Some(pos) = body.get(search_from..).and_then(|s| s.find(needle)) {
        let abs_pos = search_from.saturating_add(pos);

        let is_word_boundary = abs_pos == 0
            || body
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

        if is_word_boundary {
            count = count.saturating_add(1);
        }

        search_from = abs_pos.saturating_add(needle.len());
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MaxExpects::new())];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_too_many_expects() {
        let source = r"
test('too many', () => {
    expect(1).toBe(1);
    expect(2).toBe(2);
    expect(3).toBe(3);
    expect(4).toBe(4);
    expect(5).toBe(5);
    expect(6).toBe(6);
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "test with 6 expects (> 5) should be flagged"
        );
    }

    #[test]
    fn test_allows_within_limit() {
        let source = r"
test('ok', () => {
    expect(1).toBe(1);
    expect(2).toBe(2);
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "test with 2 expects should not be flagged"
        );
    }

    #[test]
    fn test_allows_exactly_five() {
        let source = r"
test('five', () => {
    expect(1).toBe(1);
    expect(2).toBe(2);
    expect(3).toBe(3);
    expect(4).toBe(4);
    expect(5).toBe(5);
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "test with exactly 5 expects should not be flagged"
        );
    }
}
