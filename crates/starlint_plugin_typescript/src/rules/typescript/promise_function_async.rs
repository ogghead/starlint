//! Rule: `typescript/promise-function-async`
//!
//! Require functions that return a `Promise` to be marked `async`. When a
//! function has an explicit `: Promise<...>` return type annotation but is
//! not declared `async`, it creates an inconsistency — readers expect
//! `async` functions to return promises and non-`async` functions to return
//! synchronous values.
//!
//! This rule scans source text for function declarations and expressions
//! that have `: Promise<` in their return type but lack the `async` keyword.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags functions with `: Promise<...>` return type that are not `async`.
#[derive(Debug)]
pub struct PromiseFunctionAsync;

impl LintRule for PromiseFunctionAsync {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/promise-function-async".to_owned(),
            description: "Require functions returning `Promise` to be marked `async`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains(": Promise<")
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let findings = find_non_async_promise_functions(ctx.source_text());

        for (start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/promise-function-async".to_owned(),
                message: "Functions that return a `Promise` should be marked `async`".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Scan source text for function declarations/expressions with `: Promise<`
/// return types that are not preceded by the `async` keyword.
///
/// The heuristic works line-by-line: it looks for lines containing both
/// `function ` (or `=> {` for arrow functions) and `: Promise<`, then checks
/// whether the line also contains the `async` keyword before the function
/// keyword.
///
/// Returns a list of `(start_offset, end_offset)` tuples pointing at the
/// `: Promise<` annotation.
fn find_non_async_promise_functions(source: &str) -> Vec<(u32, u32)> {
    const PROMISE_MARKER: &str = ": Promise<";

    let mut results = Vec::new();
    let mut line_start: usize = 0;

    for line in source.split('\n') {
        // Look for `: Promise<` in the line
        if let Some(promise_pos) = line.find(PROMISE_MARKER) {
            let text_before_promise = line.get(..promise_pos).unwrap_or("");

            // Check if this line defines a function (named or arrow)
            let is_function_line = text_before_promise.contains("function ")
                || text_before_promise.contains("function(")
                || line.contains("=>");

            if is_function_line {
                // Check if `async` appears before the function keyword
                let has_async = if let Some(func_pos) = text_before_promise.find("function") {
                    text_before_promise
                        .get(..func_pos)
                        .unwrap_or("")
                        .contains("async")
                } else {
                    // Arrow function: check if `async` appears before `=>`
                    text_before_promise.contains("async")
                };

                if !has_async {
                    let absolute_pos = line_start.saturating_add(promise_pos);
                    let end_pos = absolute_pos.saturating_add(PROMISE_MARKER.len());
                    let start = u32::try_from(absolute_pos).unwrap_or(0);
                    let end = u32::try_from(end_pos).unwrap_or(start);
                    results.push((start, end));
                }
            }
        }

        line_start = line_start.saturating_add(line.len()).saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PromiseFunctionAsync)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_non_async_function_returning_promise() {
        let diags = lint("function foo(): Promise<number> { return Promise.resolve(1); }");
        assert_eq!(
            diags.len(),
            1,
            "non-async function with Promise return type should be flagged"
        );
    }

    #[test]
    fn test_flags_non_async_arrow_returning_promise() {
        let diags = lint("const foo = (): Promise<void> => { return Promise.resolve(); };");
        assert_eq!(
            diags.len(),
            1,
            "non-async arrow function with Promise return type should be flagged"
        );
    }

    #[test]
    fn test_allows_async_function_returning_promise() {
        let diags = lint("async function foo(): Promise<number> { return 1; }");
        assert!(
            diags.is_empty(),
            "async function with Promise return type should not be flagged"
        );
    }

    #[test]
    fn test_allows_async_arrow_returning_promise() {
        let diags = lint("const foo = async (): Promise<void> => {};");
        assert!(
            diags.is_empty(),
            "async arrow with Promise return type should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_without_promise_return() {
        let diags = lint("function foo(): number { return 1; }");
        assert!(
            diags.is_empty(),
            "function without Promise return type should not be flagged"
        );
    }
}
