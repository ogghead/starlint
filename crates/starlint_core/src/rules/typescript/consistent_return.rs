//! Rule: `typescript/consistent-return`
//!
//! Require consistent return statements in functions. A function should either
//! always return a value or never return a value — mixing `return value;` and
//! bare `return;` is confusing and often indicates a logic error.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans function bodies for a mix of `return expr;`
//! and bare `return;` statements.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/consistent-return";

/// Flags functions that mix bare `return;` with `return value;` statements.
#[derive(Debug)]
pub struct ConsistentReturn;

impl NativeRule for ConsistentReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require consistent return statements in functions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let violations = find_inconsistent_returns(source);

        for span in violations {
            ctx.report_warning(
                RULE_NAME,
                "Function has inconsistent return statements — some return a value and some do not",
                span,
            );
        }
    }
}

/// Represents the kind of return statement found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReturnKind {
    /// A bare `return;` with no value.
    Bare,
    /// A `return <value>;` with a value.
    WithValue,
}

/// Information about a function body found in the source.
struct FunctionBody {
    /// Byte offset of the function keyword.
    start: usize,
    /// Byte offset of the end of the function's first line.
    end: usize,
    /// The opening brace position.
    brace_open: usize,
    /// The matching closing brace position.
    brace_close: usize,
}

/// Find functions that have inconsistent return statements.
///
/// Scans for `function` declarations and arrow functions, extracts their
/// bodies, and checks whether they mix bare `return;` with `return value;`.
///
/// Returns a list of spans for each function with inconsistent returns.
fn find_inconsistent_returns(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let bodies = find_function_bodies(source);

    for body in &bodies {
        let body_source = source
            .get(body.brace_open..body.brace_close.saturating_add(1))
            .unwrap_or("");

        let return_kinds = classify_returns(body_source);
        let has_bare = return_kinds.contains(&ReturnKind::Bare);
        let has_value = return_kinds.contains(&ReturnKind::WithValue);

        if has_bare && has_value {
            let start_offset = u32::try_from(body.start).unwrap_or(0);
            let end_offset = u32::try_from(body.end).unwrap_or(start_offset);
            results.push(Span::new(start_offset, end_offset));
        }
    }

    results
}

/// Find function bodies in the source text.
///
/// Looks for `function` keywords and finds the matching brace pair.
fn find_function_bodies(source: &str) -> Vec<FunctionBody> {
    let mut bodies = Vec::new();
    let needle = "function";
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(needle)) {
        let abs_pos = search_from.saturating_add(pos);

        // Make sure this is a standalone `function` keyword (not part of another word)
        let before_ok = abs_pos == 0
            || source
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

        let after_pos = abs_pos.saturating_add(needle.len());
        let after_ok = source
            .as_bytes()
            .get(after_pos)
            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

        if before_ok && after_ok {
            // Find the opening brace
            if let Some(brace_rel) = source.get(after_pos..).and_then(|s| s.find('{')) {
                let brace_open = after_pos.saturating_add(brace_rel);
                if let Some(brace_close) = find_matching_brace(source, brace_open) {
                    // Find end of the first line for the span
                    let line_end = source
                        .get(abs_pos..)
                        .and_then(|s| s.find('\n'))
                        .map_or(source.len(), |p| abs_pos.saturating_add(p));

                    bodies.push(FunctionBody {
                        start: abs_pos,
                        end: line_end,
                        brace_open,
                        brace_close,
                    });
                }
            }
        }

        search_from = abs_pos.saturating_add(1);
    }

    bodies
}

/// Find the matching closing brace for an opening brace at `open_pos`.
///
/// Tracks nesting depth to find the correct match.
fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut string_char: char = '"';
    let mut prev_char: char = '\0';

    for (idx, ch) in source.get(open_pos..)?.char_indices() {
        if in_string {
            if ch == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '{' => {
                depth = depth.saturating_add(1);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_pos.saturating_add(idx));
                }
            }
            _ => {}
        }
        prev_char = ch;
    }

    None
}

/// Classify return statements in a function body as bare or with-value.
///
/// A bare return is `return;` or `return` followed by `}` or end-of-line.
/// A value return is `return <something>;`.
fn classify_returns(body: &str) -> Vec<ReturnKind> {
    let mut kinds = Vec::new();
    let keyword = "return";
    let mut search_from: usize = 0;

    while let Some(pos) = body.get(search_from..).and_then(|s| s.find(keyword)) {
        let abs_pos = search_from.saturating_add(pos);
        let after_return = abs_pos.saturating_add(keyword.len());

        // Make sure `return` is a keyword, not part of an identifier
        let before_ok = abs_pos == 0
            || body
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

        let after_ok = body
            .as_bytes()
            .get(after_return)
            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

        if before_ok && after_ok {
            let rest = body.get(after_return..).unwrap_or("").trim_start();

            if rest.starts_with(';') || rest.starts_with('}') || rest.is_empty() {
                kinds.push(ReturnKind::Bare);
            } else {
                kinds.push(ReturnKind::WithValue);
            }
        }

        search_from = after_return;
    }

    kinds
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_mixed_returns() {
        let source = r"
function foo(x: number) {
    if (x > 0) {
        return x;
    }
    return;
}
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "function with mixed return types should be flagged"
        );
    }

    #[test]
    fn test_allows_all_value_returns() {
        let source = r"
function foo(x: number) {
    if (x > 0) {
        return x;
    }
    return 0;
}
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "function with all value returns should not be flagged"
        );
    }

    #[test]
    fn test_allows_all_bare_returns() {
        let source = r"
function foo(x: number) {
    if (x > 0) {
        return;
    }
    return;
}
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "function with all bare returns should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_return() {
        let source = r"
function foo(x: number) {
    return x;
}
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "function with a single return should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_returns() {
        let source = r"
function foo(x: number) {
    console.log(x);
}
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "function with no return statements should not be flagged"
        );
    }
}
