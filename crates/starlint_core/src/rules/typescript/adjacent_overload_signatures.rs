//! Rule: `typescript/adjacent-overload-signatures`
//!
//! Flags overloaded function signatures that are not grouped together. In
//! TypeScript, overload signatures for the same function name should appear
//! consecutively. When other declarations are interspersed between overloads
//! the code becomes harder to read and may confuse tooling.
//!
//! Uses a text-based approach: scans lines for `function name(` patterns and
//! reports when signatures with the same name are separated by other
//! declarations.

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags overloaded function signatures that are not adjacent.
#[derive(Debug)]
pub struct AdjacentOverloadSignatures;

impl NativeRule for AdjacentOverloadSignatures {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/adjacent-overload-signatures".to_owned(),
            description: "Require overload signatures to be adjacent".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let violations = find_non_adjacent_overloads(source);

        for (name, start, end) in violations {
            ctx.report(Diagnostic {
                rule_name: "typescript/adjacent-overload-signatures".to_owned(),
                message: format!("All `{name}` overload signatures should be adjacent"),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// A function signature occurrence with its name and position.
struct FuncSignature {
    /// The function name extracted from the signature line.
    name: String,
    /// Byte offset of the start of this signature in the source text.
    byte_start: usize,
    /// Byte offset of the end of this signature in the source text.
    byte_end: usize,
}

/// Scan source for function signatures and find names whose declarations are
/// not adjacent.
///
/// Returns `(function_name, span_start, span_end)` for each non-adjacent
/// occurrence of a previously-seen name.
fn find_non_adjacent_overloads(source: &str) -> Vec<(String, u32, u32)> {
    let signatures = collect_function_signatures(source);

    if signatures.len() < 2 {
        return Vec::new();
    }

    // Track the last line index where each function name appeared.
    let mut last_seen: HashMap<&str, usize> = HashMap::new();
    // Track which names have appeared at least once (to detect overloads).
    let mut results = Vec::new();

    for (i, sig) in signatures.iter().enumerate() {
        if let Some(&prev_index) = last_seen.get(sig.name.as_str()) {
            // Check if there are other function signatures between the two
            // occurrences of this name.
            let has_intervening = signatures
                .get(prev_index.saturating_add(1)..i)
                .unwrap_or(&[])
                .iter()
                .any(|s| s.name != sig.name);

            if has_intervening {
                let start = u32::try_from(sig.byte_start).unwrap_or(0);
                let end = u32::try_from(sig.byte_end).unwrap_or(start);
                results.push((sig.name.clone(), start, end));
            }
        }
        last_seen.insert(&sig.name, i);
    }

    results
}

/// Collect function signature declarations from source text.
///
/// Matches patterns like `function foo(`, `function foo<`, `export function foo(`,
/// and TypeScript method signatures in interfaces/classes.
fn collect_function_signatures(source: &str) -> Vec<FuncSignature> {
    let mut signatures = Vec::new();
    let mut byte_offset: usize = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if let Some(name) = extract_function_name(trimmed) {
            signatures.push(FuncSignature {
                name,
                byte_start: byte_offset,
                byte_end: byte_offset.saturating_add(line.len()),
            });
        }

        byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
    }

    signatures
}

/// Extract a function name from a line that looks like a function declaration
/// or overload signature.
///
/// Handles:
/// - `function foo(`
/// - `export function foo(`
/// - `declare function foo(`
/// - `foo(` (method signature in interface/class)
/// - `foo<T>(` (generic method signature)
fn extract_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    // Skip blank lines, comments, and closing braces
    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('}')
    {
        return None;
    }

    // Strip leading modifiers: export, declare, async
    let after_export = trimmed.strip_prefix("export ").unwrap_or(trimmed);
    let after_declare = after_export
        .strip_prefix("declare ")
        .unwrap_or(after_export);
    let after_modifiers = after_declare
        .strip_prefix("async ")
        .unwrap_or(after_declare)
        .trim_start();

    // `function name(...)` pattern
    if let Some(after_fn) = after_modifiers.strip_prefix("function ") {
        let after_fn_trimmed = after_fn.trim_start();
        let name: String = after_fn_trimmed
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty()
            && after_fn_trimmed.get(name.len()..).is_none_or(|tail| {
                let tail_trimmed = tail.trim_start();
                tail_trimmed.starts_with('(') || tail_trimmed.starts_with('<')
            })
        {
            return Some(name);
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AdjacentOverloadSignatures)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_non_adjacent_overloads() {
        let source = "\
function foo(a: string): void;
function bar(b: number): void;
function foo(a: number): void;
function foo(a: string | number): void {}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent overload signatures should be flagged once"
        );
    }

    #[test]
    fn test_allows_adjacent_overloads() {
        let source = "\
function foo(a: string): void;
function foo(a: number): void;
function foo(a: string | number): void {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "adjacent overload signatures should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_function() {
        let source = "function foo(a: string): void {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "a single function declaration should not be flagged"
        );
    }

    #[test]
    fn test_flags_exported_non_adjacent() {
        let source = "\
export function process(a: string): void;
export function transform(b: number): void;
export function process(a: number): void;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent exported overloads should be flagged"
        );
    }

    #[test]
    fn test_allows_different_functions() {
        let source = "\
function alpha(a: string): void;
function beta(b: number): void;
function gamma(c: boolean): void;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "different function names should not be flagged"
        );
    }
}
