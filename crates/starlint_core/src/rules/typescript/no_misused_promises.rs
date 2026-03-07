//! Rule: `typescript/no-misused-promises`
//!
//! Disallow promises in places where they are not properly handled. Flags
//! promise-returning functions used as void callbacks, such as passing an
//! `async` function to `addEventListener`.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans for `addEventListener` calls whose callback
//! argument references a function declared as `async` in the same file.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-misused-promises";

/// Flags async functions passed as void callbacks (e.g. to `addEventListener`).
#[derive(Debug)]
pub struct NoMisusedPromises;

impl LintRule for NoMisusedPromises {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow promises in places where they are not properly handled"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();

        // Phase 1: collect names of functions declared with `async function <name>`.
        let async_fn_names = collect_async_function_names(source);
        if async_fn_names.is_empty() {
            return;
        }

        // Phase 2: scan for addEventListener calls passing an async function name.
        let violations = find_misused_promise_callbacks(source, &async_fn_names);

        for (span, fn_name) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Promise-returning function `{fn_name}` passed as a void callback — \
                     its returned promise will not be handled"
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

/// Find `addEventListener` calls where the callback argument is the name of an
/// async function. Returns `(span, function_name)` pairs.
fn find_misused_promise_callbacks(source: &str, async_fn_names: &[String]) -> Vec<(Span, String)> {
    let mut results = Vec::new();

    // Pattern: addEventListener("...", <asyncFnName>)
    // We look for `addEventListener(` and then inspect the second argument.
    let needle = "addEventListener(";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(pos);
        let args_start = abs_pos.saturating_add(needle.len());

        // Find the matching closing paren (simple: first `)` after the opening).
        let call_end = source
            .get(args_start..)
            .and_then(|s| s.find(')'))
            .map(|p| args_start.saturating_add(p));

        if let Some(end_pos) = call_end {
            let args_text = source.get(args_start..end_pos).unwrap_or("");

            // Split on the first comma to get the second argument.
            if let Some(comma_pos) = args_text.find(',') {
                let second_arg = args_text
                    .get(comma_pos.saturating_add(1)..)
                    .unwrap_or("")
                    .trim();

                for name in async_fn_names {
                    if second_arg == name.as_str() {
                        let start = u32::try_from(abs_pos).unwrap_or(0);
                        let end = u32::try_from(end_pos.saturating_add(1)).unwrap_or(start);
                        results.push((Span::new(start, end), name.clone()));
                    }
                }
            }
        }

        search_start = abs_pos.saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMisusedPromises)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_async_fn_as_event_listener() {
        let source = r#"
async function handleClick() { await fetch("/api"); }
document.addEventListener("click", handleClick);
"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "async function passed to addEventListener should be flagged"
        );
    }

    #[test]
    fn test_allows_sync_fn_as_event_listener() {
        let source = r#"
function handleClick() { console.log("clicked"); }
document.addEventListener("click", handleClick);
"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "synchronous function passed to addEventListener should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_add_event_listener() {
        let source = r#"
async function fetchData() { return await fetch("/api"); }
const result = await fetchData();
"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "async function not used as callback should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_async_callbacks() {
        let source = r#"
async function onLoad() { await init(); }
async function onScroll() { await update(); }
window.addEventListener("load", onLoad);
window.addEventListener("scroll", onScroll);
"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            2,
            "both async callbacks passed to addEventListener should be flagged"
        );
    }

    #[test]
    fn test_allows_inline_arrow_fn() {
        let source = r#"
document.addEventListener("click", () => { console.log("hi"); });
"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "inline arrow functions should not be flagged"
        );
    }
}
