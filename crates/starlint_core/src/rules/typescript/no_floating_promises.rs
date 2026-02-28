//! Rule: `typescript/no-floating-promises`
//!
//! Require promises to be handled. Flags expression statements that are bare
//! call expressions to functions declared with `async` in the same file,
//! where the call is not awaited, not chained with `.then()`/`.catch()`,
//! and not assigned to a variable.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans for `async function` declarations and then
//! flags standalone calls to those functions.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-floating-promises";

/// Flags unhandled calls to functions declared as `async` in the same file.
#[derive(Debug)]
pub struct NoFloatingPromises;

impl NativeRule for NoFloatingPromises {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require promises to be handled".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();

        // Phase 1: collect names of functions declared with `async function <name>`.
        let async_fn_names = collect_async_function_names(source);
        if async_fn_names.is_empty() {
            return;
        }

        // Phase 2: scan for standalone call expression statements to those names.
        let violations = find_floating_calls(source, &async_fn_names);

        for span in violations {
            ctx.report_warning(
                RULE_NAME,
                "Promises must be awaited, returned, or handled with `.then()`/`.catch()`",
                span,
            );
        }
    }
}

/// Collect names of functions declared as `async function <name>`.
fn collect_async_function_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let needle = "async function ";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(pos);
        let name_start = abs_pos.saturating_add(needle.len());

        // Extract the function name (sequence of word characters).
        let name: String = source
            .get(name_start..)
            .unwrap_or("")
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
            .collect();

        if !name.is_empty() {
            names.push(name);
        }

        search_start = name_start.saturating_add(1);
    }

    names
}

/// Find standalone call statements to the given async function names.
/// A "standalone call" is a trimmed line that starts with `<name>(` and is not
/// preceded by `await `, `return `, or an assignment (`=`), and is not
/// followed by `.then(` or `.catch(`.
fn find_floating_calls(source: &str, async_fn_names: &[String]) -> Vec<Span> {
    let mut results = Vec::new();
    let mut byte_offset: u32 = 0;

    for line in source.lines() {
        let line_len = u32::try_from(line.len()).unwrap_or(0);
        let trimmed = line.trim();

        for name in async_fn_names {
            let call_prefix = format!("{name}(");

            if trimmed.starts_with(&call_prefix) {
                // Exclude lines that are awaited, returned, assigned, or chained.
                let is_handled = trimmed.contains(".then(")
                    || trimmed.contains(".catch(");

                if !is_handled {
                    let offset_in_line =
                        u32::try_from(line.len().saturating_sub(trimmed.len())).unwrap_or(0);
                    let start = byte_offset.saturating_add(offset_in_line);
                    let end = byte_offset.saturating_add(line_len);
                    results.push(Span::new(start, end));
                }
            }
        }

        // +1 for the newline character
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    results
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoFloatingPromises)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_floating_async_call() {
        let source = "async function fetchData() { return 1; }\nfetchData();";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "standalone call to async function should be flagged"
        );
    }

    #[test]
    fn test_allows_then_chained_call() {
        let source =
            "async function fetchData() { return 1; }\nfetchData().then(x => x);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "call chained with .then() should not be flagged"
        );
    }

    #[test]
    fn test_allows_catch_chained_call() {
        let source =
            "async function fetchData() { return 1; }\nfetchData().catch(e => e);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "call chained with .catch() should not be flagged"
        );
    }

    #[test]
    fn test_allows_awaited_call() {
        let source = "async function fetchData() { return 1; }\nasync function main() { await fetchData(); }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "awaited call should not be flagged"
        );
    }

    #[test]
    fn test_no_async_functions_no_flags() {
        let source = "function syncFunc() { return 1; }\nsyncFunc();";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "calls to non-async functions should not be flagged"
        );
    }
}
