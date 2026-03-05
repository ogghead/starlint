//! Rule: `typescript/require-await`
//!
//! Disallow async functions that have no `await` expression. An async
//! function without `await` is misleading because the reader expects
//! asynchronous operations inside it. Either add an `await`, remove the
//! `async` keyword, or restructure the code.
//!
//! This rule uses text-based scanning via `run_once()` to find `async`
//! function declarations and arrow functions, then checks whether their
//! body contains an `await` expression.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags async functions that contain no `await` expressions.
#[derive(Debug)]
pub struct RequireAwait;

impl NativeRule for RequireAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/require-await".to_owned(),
            description: "Disallow async functions which have no await expression".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_async_without_await(source);

        for (start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/require-await".to_owned(),
                message: "Async function has no `await` expression".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Find `async function` declarations and `async () =>` arrow functions
/// whose body does not contain an `await` keyword.
///
/// Returns `(start, end)` byte offsets for the `async` keyword of each
/// flagged function.
fn find_async_without_await(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find("async")) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_async = absolute_pos.saturating_add("async".len());

        // Make sure `async` is a standalone keyword (not part of another identifier)
        let before_ok = absolute_pos == 0
            || source
                .as_bytes()
                .get(absolute_pos.saturating_sub(1))
                .is_none_or(|&b| !b.is_ascii_alphanumeric() && b != b'_');
        let after_ok = source
            .as_bytes()
            .get(after_async)
            .is_none_or(|&b| !b.is_ascii_alphanumeric() && b != b'_');

        if !before_ok || !after_ok {
            search_from = after_async;
            continue;
        }

        let rest = source.get(after_async..).unwrap_or("").trim_start();

        // Match `async function` or `async (` / `async () =>`
        let is_async_func = rest.starts_with("function") || rest.starts_with('(');
        if !is_async_func {
            search_from = after_async;
            continue;
        }

        // Find the body: locate the opening `{` after the async keyword
        let Some(body_brace_offset) = source.get(after_async..).and_then(|s| s.find('{')) else {
            search_from = after_async;
            continue;
        };
        let body_start = after_async.saturating_add(body_brace_offset);

        let Some(body_end) = find_matching_brace(source, body_start) else {
            search_from = after_async;
            continue;
        };

        let body = source
            .get(body_start.saturating_add(1)..body_end)
            .unwrap_or("");

        if !body.contains("await") {
            let start = u32::try_from(absolute_pos).unwrap_or(0);
            let end = u32::try_from(after_async).unwrap_or(start);
            results.push((start, end));
        }

        search_from = body_end.saturating_add(1);
    }

    results
}

/// Find the position of the matching closing brace for an opening `{`.
fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    let mut depth: u32 = 0;
    for (i, ch) in source.get(open_pos..)?.char_indices() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_pos.saturating_add(i));
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_async_function_without_await() {
        let diags = lint("async function foo() { return 1; }");
        assert_eq!(
            diags.len(),
            1,
            "async function without await should be flagged"
        );
    }

    #[test]
    fn test_allows_async_function_with_await() {
        let diags = lint("async function foo() { await bar(); }");
        assert!(
            diags.is_empty(),
            "async function with await should not be flagged"
        );
    }

    #[test]
    fn test_flags_async_arrow_without_await() {
        let diags = lint("const foo = async () => { return 1; };");
        assert_eq!(
            diags.len(),
            1,
            "async arrow without await should be flagged"
        );
    }

    #[test]
    fn test_allows_async_arrow_with_await() {
        let diags = lint("const foo = async () => { await bar(); };");
        assert!(
            diags.is_empty(),
            "async arrow with await should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_async_function() {
        let diags = lint("function foo() { return 1; }");
        assert!(diags.is_empty(), "non-async function should not be flagged");
    }
}
