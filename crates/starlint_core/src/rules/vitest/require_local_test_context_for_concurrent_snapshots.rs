//! Rule: `vitest/require-local-test-context-for-concurrent-snapshots`
//!
//! Error when concurrent tests use snapshot matchers without local test context.
//! When using `test.concurrent`, snapshot matchers like `toMatchSnapshot()`
//! require accessing `expect` from the test context parameter (e.g.
//! `test.concurrent("name", ({ expect }) => { ... })`) rather than the global
//! `expect`. This is because concurrent tests run in parallel and the global
//! `expect` cannot track snapshots correctly across concurrent executions.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/require-local-test-context-for-concurrent-snapshots";

/// Snapshot matchers that require local test context in concurrent tests.
const SNAPSHOT_MATCHERS: &[&str] = &[
    "toMatchSnapshot",
    "toMatchInlineSnapshot",
    "toThrowErrorMatchingSnapshot",
    "toThrowErrorMatchingInlineSnapshot",
];

/// Error when concurrent tests use snapshots without local context.
#[derive(Debug)]
pub struct RequireLocalTestContextForConcurrentSnapshots;

impl NativeRule for RequireLocalTestContextForConcurrentSnapshots {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require local test context for snapshot matchers in concurrent tests"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = find_concurrent_snapshot_violations(ctx.source_text());

        for (span, matcher) in violations {
            ctx.report_error(
                RULE_NAME,
                &format!(
                    "Concurrent test uses `{matcher}` without local test context — destructure `{{ expect }}` from the test context parameter"
                ),
                span,
            );
        }
    }
}

/// Find concurrent test blocks that use snapshot matchers without local context.
/// Returns `(span, matcher_name)` pairs.
fn find_concurrent_snapshot_violations(source: &str) -> Vec<(Span, String)> {
    let mut results = Vec::new();
    let patterns = &["test.concurrent(", "it.concurrent("];

    for pattern in patterns {
        let mut search_start: usize = 0;
        while let Some(pos) = source.get(search_start..).and_then(|s| s.find(pattern)) {
            let abs_pos = search_start.saturating_add(pos);
            let after_call = abs_pos.saturating_add(pattern.len());

            if let Some(rest) = source.get(after_call..) {
                if let Some(brace_offset) = rest.find('{') {
                    let body_start = after_call.saturating_add(brace_offset);

                    let sig_region = source.get(after_call..body_start).unwrap_or("");
                    let has_local_expect =
                        sig_region.contains("{ expect") || sig_region.contains("{expect");

                    if let Some(body_end) = find_matching_brace(source, body_start) {
                        let body = source.get(body_start..body_end).unwrap_or("");

                        for matcher in SNAPSHOT_MATCHERS {
                            if body.contains(matcher) && !has_local_expect {
                                let start = u32::try_from(abs_pos).unwrap_or(0);
                                let end = u32::try_from(abs_pos.saturating_add(pattern.len()))
                                    .unwrap_or(start);
                                results.push((Span::new(start, end), (*matcher).to_owned()));
                                break;
                            }
                        }
                    }
                }
            }

            search_start = abs_pos.saturating_add(1);
        }
    }

    results
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> =
                vec![Box::new(RequireLocalTestContextForConcurrentSnapshots)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_concurrent_snapshot_without_context() {
        let source = r#"test.concurrent("my test", () => { expect(value).toMatchSnapshot(); });"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "concurrent test with snapshot and no local context should be flagged"
        );
    }

    #[test]
    fn test_allows_concurrent_snapshot_with_context() {
        let source =
            r#"test.concurrent("my test", ({ expect }) => { expect(value).toMatchSnapshot(); });"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "concurrent test with local `expect` should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_concurrent_snapshot() {
        let source = r#"test("my test", () => { expect(value).toMatchSnapshot(); });"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-concurrent test with snapshot should not be flagged"
        );
    }
}
