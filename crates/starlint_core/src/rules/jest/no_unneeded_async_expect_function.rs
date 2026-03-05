//! Rule: `jest/no-unneeded-async-expect-function`
//!
//! Warn when an async test function body only contains `await expect(...)`.
//! Simplified: flags `it`/`test` calls whose callback is `async` but the body
//! only has a single `await expect(...)` statement.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-unneeded-async-expect-function";

/// Flags `it`/`test` with async callbacks that only contain `await expect`.
#[derive(Debug)]
pub struct NoUnneededAsyncExpectFunction;

impl NativeRule for NoUnneededAsyncExpectFunction {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary async in test functions that only await expect"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            let test_names = ["it(", "test("];
            let mut violations: Vec<Span> = Vec::new();

            for test_name in &test_names {
                let mut search_start: usize = 0;

                while let Some(pos) = source.get(search_start..).and_then(|s| s.find(test_name)) {
                    let abs_pos = search_start.saturating_add(pos);

                    if is_async_single_await_expect(source, abs_pos, test_name) {
                        let span_end = find_matching_close_paren(source, abs_pos)
                            .unwrap_or_else(|| abs_pos.saturating_add(test_name.len()));

                        let start_u32 = u32::try_from(abs_pos).unwrap_or(0);
                        let end_u32 = u32::try_from(span_end).unwrap_or(start_u32);
                        violations.push(Span::new(start_u32, end_u32));
                    }

                    search_start = abs_pos.saturating_add(test_name.len());
                }
            }

            violations
        };

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "This test function does not need to be async — `await expect(...)` can be replaced with `expect(...)` and `.resolves`/`.rejects`".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a test call at `pos` has an async callback that only contains
/// a single `await expect(...)` statement.
fn is_async_single_await_expect(source: &str, pos: usize, test_name: &str) -> bool {
    let after_name = pos.saturating_add(test_name.len());
    let rest = source.get(after_name..).unwrap_or("");

    // Skip the test description string argument and comma
    let Some(comma_pos) = rest.find(',') else {
        return false;
    };
    let after_comma = rest.get(comma_pos.saturating_add(1)..).unwrap_or("");
    let trimmed = after_comma.trim_start();

    // Check if the callback starts with `async`
    if !trimmed.starts_with("async") {
        return false;
    }

    // Find the function body (between `{` and `}`)
    let Some(brace_start) = trimmed.find('{') else {
        return false;
    };
    let body_start = brace_start.saturating_add(1);
    let body_rest = trimmed.get(body_start..).unwrap_or("");

    // Find the matching closing brace
    let mut depth: usize = 1;
    let mut body_end: usize = 0;
    for (i, ch) in body_rest.char_indices() {
        if ch == '{' {
            depth = depth.saturating_add(1);
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                body_end = i;
                break;
            }
        }
    }

    if body_end == 0 {
        return false;
    }

    let body = body_rest.get(..body_end).unwrap_or("").trim();

    // Check if the body only contains `await expect(...)`
    // Simple heuristic: body starts with "await expect(" and contains only one statement
    let is_single_await_expect = body.starts_with("await expect(") && !body.contains('\n')
        || (body.lines().count() == 1 && body.trim().starts_with("await expect("));

    // Also handle multiline but still single statement
    if !is_single_await_expect {
        let lines: Vec<&str> = body
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if lines.len() == 1 {
            return lines
                .first()
                .is_some_and(|l| l.starts_with("await expect("));
        }
        return false;
    }

    true
}

/// Find the matching closing parenthesis for a test call.
fn find_matching_close_paren(source: &str, start: usize) -> Option<usize> {
    let rest = source.get(start..)?;
    let open_pos = rest.find('(')?;
    let after_open = open_pos.saturating_add(1);

    let mut depth: usize = 1;
    for (i, ch) in rest.get(after_open..)?.char_indices() {
        if ch == '(' {
            depth = depth.saturating_add(1);
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(
                    start
                        .saturating_add(after_open)
                        .saturating_add(i)
                        .saturating_add(1),
                );
            }
        }
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnneededAsyncExpectFunction)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_async_single_await_expect() {
        let source =
            r"it('resolves', async () => { await expect(fetchData()).resolves.toBe(1); });";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "async test with only `await expect(...)` should be flagged"
        );
    }

    #[test]
    fn test_allows_async_with_multiple_statements() {
        let source = r"
it('does stuff', async () => {
    const data = await fetchData();
    expect(data).toBe(1);
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "async test with multiple statements should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_async_test() {
        let source = r"it('sync', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert!(diags.is_empty(), "non-async test should not be flagged");
    }
}
