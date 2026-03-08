//! Rule: `jest/require-hook`
//!
//! Warn when setup code is placed directly in `describe` blocks instead of
//! inside lifecycle hooks. Only calls to `it`/`test`/`describe`/`beforeEach`/
//! `afterEach`/`beforeAll`/`afterAll` are expected at the top level of a
//! `describe` body.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/require-hook";

/// Allowed top-level calls inside `describe` blocks.
const ALLOWED_CALLS: &[&str] = &[
    "it(",
    "test(",
    "describe(",
    "beforeEach(",
    "afterEach(",
    "beforeAll(",
    "afterAll(",
    "it.each(",
    "test.each(",
    "describe.each(",
    "it.skip(",
    "test.skip(",
    "describe.skip(",
    "it.only(",
    "test.only(",
    "describe.only(",
    "it.todo(",
    "test.todo(",
];

/// Flags expression statements inside `describe` blocks that are not
/// test/hook/`describe` calls.
#[derive(Debug)]
pub struct RequireHook;

impl LintRule for RequireHook {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require setup code to be in lifecycle hooks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("describe(") && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = {
            let source = ctx.source_text();

            // Early exit: skip files without describe blocks.
            if !source.contains("describe(") {
                return;
            }

            let needle = "describe(";
            let mut violations: Vec<Span> = Vec::new();

            let mut search_start: usize = 0;

            while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
                let abs_pos = search_start.saturating_add(pos);

                let is_word_boundary = abs_pos == 0
                    || source
                        .as_bytes()
                        .get(abs_pos.saturating_sub(1))
                        .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

                if is_word_boundary {
                    collect_describe_violations(source, abs_pos, &mut violations);
                }

                search_start = abs_pos.saturating_add(needle.len());
            }

            violations
        };

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Move setup code into a lifecycle hook (`beforeEach`, `beforeAll`, etc.)"
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

/// Collect violations from the direct statements inside a `describe` callback body.
fn collect_describe_violations(source: &str, describe_pos: usize, violations: &mut Vec<Span>) {
    let rest = source.get(describe_pos..).unwrap_or("");

    // Find the callback body
    let Some(brace_pos) = rest.find('{') else {
        return;
    };
    let body_start = brace_pos.saturating_add(1);
    let body_rest = rest.get(body_start..).unwrap_or("");

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
        return;
    }

    let body = body_rest.get(..body_end).unwrap_or("");

    // Walk through the body line by line, checking top-level statements.
    // We only flag lines at nesting depth 0 (relative to the describe body).
    let mut brace_depth: usize = 0;
    let mut byte_offset: usize = 0;

    for line in body.lines() {
        let trimmed = line.trim();
        let line_len = line.len();

        // Only check statements at the top level of the describe body
        if brace_depth == 0 && !trimmed.is_empty() {
            // Check if this line is an allowed call
            let is_allowed = ALLOWED_CALLS.iter().any(|pat| trimmed.starts_with(pat))
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
                || trimmed.starts_with("const ")
                || trimmed.starts_with("let ")
                || trimmed.starts_with("var ")
                || trimmed == "})"
                || trimmed == "});"
                || trimmed == "}"
                || trimmed == "};";

            if !is_allowed && !trimmed.is_empty() {
                let abs_start = describe_pos
                    .saturating_add(body_start)
                    .saturating_add(byte_offset)
                    .saturating_add(line.len().saturating_sub(trimmed.len()));
                let abs_end = abs_start.saturating_add(trimmed.len());

                let start_u32 = u32::try_from(abs_start).unwrap_or(0);
                let end_u32 = u32::try_from(abs_end).unwrap_or(start_u32);

                violations.push(Span::new(start_u32, end_u32));
            }
        }

        // Track brace depth
        for ch in line.chars() {
            if ch == '{' {
                brace_depth = brace_depth.saturating_add(1);
            } else if ch == '}' {
                brace_depth = brace_depth.saturating_sub(1);
            }
        }

        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireHook)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_setup_outside_hook() {
        let source = r"
describe('suite', () => {
    jest.useFakeTimers();
    test('works', () => { expect(1).toBe(1); });
});
";
        let diags = lint(source);
        assert!(
            !diags.is_empty(),
            "setup code directly in describe should be flagged"
        );
    }

    #[test]
    fn test_allows_setup_in_hook() {
        let source = r"
describe('suite', () => {
    beforeEach(() => {
        jest.useFakeTimers();
    });
    test('works', () => { expect(1).toBe(1); });
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "setup code in beforeEach should not be flagged"
        );
    }

    #[test]
    fn test_allows_test_and_describe_calls() {
        let source = r"
describe('suite', () => {
    describe('nested', () => {
        test('works', () => { expect(1).toBe(1); });
    });
    it('also works', () => { expect(2).toBe(2); });
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "test and describe calls should not be flagged"
        );
    }
}
