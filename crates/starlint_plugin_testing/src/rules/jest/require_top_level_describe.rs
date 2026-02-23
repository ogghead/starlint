//! Rule: `jest/require-top-level-describe`
//!
//! Warn when `it`/`test` are used at the top level without a `describe` wrapper.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/require-top-level-describe";

/// Flags `it` and `test` calls that are not inside any `describe` block.
#[derive(Debug)]
pub struct RequireTopLevelDescribe;

impl LintRule for RequireTopLevelDescribe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `test`/`it` to be inside a `describe` block".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("it(") || source_text.contains("test("))
            && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            find_top_level_tests(source)
        };

        for (msg, span) in &violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: msg.to_owned(),
                span: *span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Single forward pass: track describe nesting via brace depth stack.
///
/// Previous implementation was O(m×n) — for each `it(`/`test(` match it
/// re-scanned all previous `describe(` blocks to check nesting.
/// This implementation is O(n) — single scan through the source.
fn find_top_level_tests(source: &str) -> Vec<(String, Span)> {
    #[derive(Clone, Copy)]
    enum TestKeyword {
        Describe,
        It,
        Test,
    }

    let describe_needle = "describe(";
    let it_needle = "it(";
    let test_needle = "test(";
    let mut violations: Vec<(String, Span)> = Vec::new();

    let mut positions: Vec<(usize, TestKeyword)> = Vec::new();

    for needle_info in &[
        (describe_needle, TestKeyword::Describe),
        (it_needle, TestKeyword::It),
        (test_needle, TestKeyword::Test),
    ] {
        let (needle, kind) = needle_info;
        let mut search_start: usize = 0;
        while let Some(offset) = source.get(search_start..).and_then(|s| s.find(needle)) {
            let abs_pos = search_start.saturating_add(offset);
            let is_word_boundary = abs_pos == 0
                || source
                    .as_bytes()
                    .get(abs_pos.saturating_sub(1))
                    .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');
            if is_word_boundary {
                positions.push((abs_pos, *kind));
            }
            search_start = abs_pos.saturating_add(needle.len());
        }
    }
    positions.sort_unstable_by_key(|&(pos, _)| pos);

    // Forward pass: count braces and track describe nesting.
    let mut describe_stack: Vec<usize> = Vec::new();
    let mut brace_depth: usize = 0;
    let mut kw_idx: usize = 0;

    // Use byte iteration since `{`, `}` are ASCII — avoids UTF-8 decoding overhead.
    let bytes = source.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'{' {
            brace_depth = brace_depth.saturating_add(1);
        } else if b == b'}' {
            brace_depth = brace_depth.saturating_sub(1);
            while describe_stack.last().is_some_and(|&d| d == brace_depth) {
                describe_stack.pop();
            }
        }

        while kw_idx < positions.len() && positions.get(kw_idx).is_some_and(|&(p, _)| p == i) {
            if let Some(&(pos, kind)) = positions.get(kw_idx) {
                match kind {
                    TestKeyword::Describe => {
                        describe_stack.push(brace_depth);
                    }
                    TestKeyword::It => {
                        if describe_stack.is_empty() {
                            let start_u32 = u32::try_from(pos).unwrap_or(0);
                            violations.push((
                                "`it` should be inside a `describe` block".to_owned(),
                                Span::new(start_u32, start_u32.saturating_add(2)),
                            ));
                        }
                    }
                    TestKeyword::Test => {
                        if describe_stack.is_empty() {
                            let start_u32 = u32::try_from(pos).unwrap_or(0);
                            violations.push((
                                "`test` should be inside a `describe` block".to_owned(),
                                Span::new(start_u32, start_u32.saturating_add(4)),
                            ));
                        }
                    }
                }
            }
            kw_idx = kw_idx.saturating_add(1);
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireTopLevelDescribe)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_top_level_test() {
        let source = "test('works', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "top-level `test` should be flagged");
    }

    #[test]
    fn test_flags_top_level_it() {
        let source = "it('works', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "top-level `it` should be flagged");
    }

    #[test]
    fn test_allows_test_inside_describe() {
        let source = r"
describe('suite', () => {
    test('works', () => { expect(1).toBe(1); });
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`test` inside `describe` should not be flagged"
        );
    }
}
