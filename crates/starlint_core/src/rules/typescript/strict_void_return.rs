//! Rule: `typescript/strict-void-return`
//!
//! Disallow returning a value from functions typed as returning `void`.
//! A function annotated with `: void` should only use bare `return;`
//! statements, not `return expr;`.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans for functions with `: void` return type
//! annotations and flags `return <expr>;` statements within them.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/strict-void-return";

/// Flags functions with `: void` return annotations that contain non-bare
/// `return` statements.
#[derive(Debug)]
pub struct StrictVoidReturn;

impl NativeRule for StrictVoidReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow returning a value from functions typed as returning void"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let violations = find_void_return_violations(source);

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not return a value from a function typed as returning `void`"
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

/// A region of source text representing a function body with a `: void` return
/// type annotation.
struct VoidFunctionRegion {
    /// Byte offset of the opening brace of the function body.
    body_start: usize,
    /// Byte offset of the closing brace of the function body.
    body_end: usize,
}

/// Find all functions annotated with `: void` and locate their body regions.
fn find_void_function_regions(source: &str) -> Vec<VoidFunctionRegion> {
    let mut regions = Vec::new();
    let needle = ": void";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(pos);
        let after_void = abs_pos.saturating_add(needle.len());

        // Look for the opening brace `{` after `: void`
        if let Some(brace_offset) = source.get(after_void..).and_then(|s| s.find('{')) {
            let brace_pos = after_void.saturating_add(brace_offset);

            // Verify that between `: void` and `{` there is only whitespace and
            // optionally `=>` (for arrow functions).
            let between = source.get(after_void..brace_pos).unwrap_or("");
            let trimmed_between = between.trim();
            let is_valid_gap = trimmed_between.is_empty() || trimmed_between == "=>";

            if is_valid_gap {
                if let Some(close_pos) = find_matching_brace(source, brace_pos) {
                    regions.push(VoidFunctionRegion {
                        body_start: brace_pos,
                        body_end: close_pos,
                    });
                }
            }
        }

        search_start = after_void;
    }

    regions
}

/// Find the matching closing brace for an opening brace at `open_pos`.
///
/// Returns the byte offset of the matching `}`, or `None` if unbalanced.
fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    let mut depth: usize = 0;
    let slice = source.get(open_pos..)?;

    for (offset, ch) in slice.char_indices() {
        if ch == '{' {
            depth = depth.saturating_add(1);
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(open_pos.saturating_add(offset));
            }
        }
    }

    None
}

/// Check whether a `return` statement at the given position is a bare `return;`
/// (no expression).
///
/// Returns `true` if the return is bare — i.e. immediately followed by `;` or
/// a newline (with only whitespace in between).
fn is_bare_return(source: &str, return_end: usize) -> bool {
    let after = source.get(return_end..).unwrap_or("");
    let trimmed = after.trim_start();
    trimmed.starts_with(';') || trimmed.starts_with('\n') || trimmed.starts_with('}')
}

/// Find all non-bare `return` statements inside `: void` function bodies.
fn find_void_return_violations(source: &str) -> Vec<Span> {
    let regions = find_void_function_regions(source);
    let mut results = Vec::new();

    for region in &regions {
        let body = source
            .get(region.body_start..=region.body_end)
            .unwrap_or("");
        let needle = "return ";

        let mut search_start: usize = 0;
        while let Some(pos) = body.get(search_start..).and_then(|s| s.find(needle)) {
            let abs_in_body = search_start.saturating_add(pos);
            let abs_in_source = region.body_start.saturating_add(abs_in_body);
            let after_return = abs_in_source.saturating_add(needle.len());

            // Ensure this is not part of a larger identifier (e.g. `returnValue`)
            let before_return = if abs_in_source > 0 {
                source
                    .get(abs_in_source.saturating_sub(1)..abs_in_source)
                    .unwrap_or("")
            } else {
                ""
            };
            let preceded_by_word_char = before_return
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_');

            if !preceded_by_word_char && !is_bare_return(source, after_return) {
                // Find the end of the return statement (semicolon or newline)
                let stmt_end = source
                    .get(abs_in_source..)
                    .and_then(|s| s.find(';'))
                    .map_or(after_return, |p| {
                        abs_in_source.saturating_add(p).saturating_add(1)
                    });

                let start = u32::try_from(abs_in_source).unwrap_or(0);
                let end = u32::try_from(stmt_end).unwrap_or(start);
                results.push(Span::new(start, end));
            }

            search_start = abs_in_body.saturating_add(needle.len());
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(StrictVoidReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_value_in_void_function() {
        let source = "function greet(name: string): void { return name; }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "returning a value from a void function should be flagged"
        );
    }

    #[test]
    fn test_allows_bare_return_in_void_function() {
        let source = "function reset(): void { return; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "bare return in a void function should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_in_non_void_function() {
        let source = "function add(a: number, b: number): number { return a + b; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "returning a value from a non-void function should not be flagged"
        );
    }

    #[test]
    fn test_flags_arrow_void_return() {
        let source = "const log = (msg: string): void => { return msg; };";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "returning a value from a void arrow function should be flagged"
        );
    }

    #[test]
    fn test_allows_void_function_without_return() {
        let source = "function doWork(): void { console.log('working'); }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "void function without return should not be flagged"
        );
    }
}
