//! Rule: `typescript/prefer-readonly-parameter-types`
//!
//! Prefer `readonly` parameter types. Flags function parameters with array
//! type annotations (e.g. `string[]`) that are not marked as `readonly`.
//! Using `readonly string[]` communicates that the function does not mutate
//! the array.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans for function parameter type annotations
//! containing `[]` that are not preceded by the `readonly` keyword.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-readonly-parameter-types";

/// Flags function parameters with mutable array type annotations that should
/// use `readonly`.
#[derive(Debug)]
pub struct PreferReadonlyParameterTypes;

impl LintRule for PreferReadonlyParameterTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer readonly parameter types for array parameters".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let violations = find_mutable_array_params(source);

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Array parameter should use `readonly` type annotation (e.g. `readonly string[]`)".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a position inside a parameter list is preceded by `readonly` within
/// the same type annotation context.
fn is_preceded_by_readonly(source: &str, bracket_pos: usize) -> bool {
    // Look backwards from `[]` for `readonly ` within a reasonable window
    let look_back = bracket_pos.min(30);
    let start = bracket_pos.saturating_sub(look_back);
    let before = source.get(start..bracket_pos).unwrap_or("");

    // Check if the text before the `[]` contains `readonly` as the type modifier
    // We look for "readonly " near the end of the `before` slice (after the colon)
    if let Some(colon_pos) = before.rfind(':') {
        let type_region = before.get(colon_pos..).unwrap_or("");
        return type_region.contains("readonly ");
    }

    false
}

/// Scan source text for function parameters with mutable `[]` array type
/// annotations.
///
/// Returns a list of [`Span`] values for each parameter that should be
/// flagged.
fn find_mutable_array_params(source: &str) -> Vec<Span> {
    let mut results = Vec::new();

    // Look for function signatures: `function`, arrow functions, method
    // signatures. We scan for `:` inside parameter lists followed by `[]`.
    let fn_keywords = ["function ", "=> {", "=> ("];

    // Strategy: find every `(` that looks like a parameter list, then scan
    // inside for `: <type>[]` patterns not preceded by `readonly`.

    // We use a simpler approach: scan for `[]` patterns that appear after `:`
    // inside function parameter contexts.
    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find("[]")) {
        let abs_pos = search_start.saturating_add(pos);

        // Check that this `[]` is part of a type annotation (preceded by `:` somewhere)
        let look_back = abs_pos.min(80);
        let region_start = abs_pos.saturating_sub(look_back);
        let before = source.get(region_start..abs_pos).unwrap_or("");

        // Verify there is a colon (type annotation) in the region before `[]`
        let has_colon = before.rfind(':').is_some();

        // Verify this is inside parentheses (parameter context) by checking
        // for an unmatched `(` before this position
        let is_in_params = is_inside_parens(source, abs_pos);

        // Ensure this is not a `readonly` array
        let has_readonly = is_preceded_by_readonly(source, abs_pos);

        // Ensure this is not an array literal like `const x = []`
        let is_type_context = has_colon && !is_array_literal(before);

        // Check that this is not inside a function keyword declaration itself
        // (e.g. not a return type annotation)
        let is_fn_return_type = is_return_type_context(source, abs_pos);

        if is_type_context && is_in_params && !has_readonly && !is_fn_return_type {
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(2);
            results.push(Span::new(start, end));
        }

        search_start = abs_pos.saturating_add(2);
    }

    // Deduplicate: use the fn_keywords check only as a broad filter
    let _ = fn_keywords;

    results
}

/// Check if a given position is inside parentheses by counting unmatched
/// opening parens before it on the same logical line.
fn is_inside_parens(source: &str, pos: usize) -> bool {
    let before = source.get(..pos).unwrap_or("");

    // Walk backwards to find if we are between `(` and `)`
    let mut depth: i32 = 0;
    for ch in before.chars().rev() {
        if ch == ')' {
            depth = depth.saturating_add(1);
        } else if ch == '(' {
            if depth > 0 {
                depth = depth.saturating_sub(1);
            } else {
                return true;
            }
        }
        // Stop at function body boundaries
        if ch == '{' || ch == '}' {
            break;
        }
    }

    false
}

/// Check if the `[]` is part of an array literal rather than a type annotation.
fn is_array_literal(before: &str) -> bool {
    let trimmed = before.trim_end();
    trimmed.ends_with('=') || trimmed.ends_with(',') || trimmed.ends_with('[')
}

/// Check if a `[]` at the given position is part of a function return type
/// rather than a parameter type.
fn is_return_type_context(source: &str, pos: usize) -> bool {
    let before = source.get(..pos).unwrap_or("");

    // Look for `)` followed by `:` (return type annotation)
    // Find the closest `)` before this position
    if let Some(close_paren) = before.rfind(')') {
        let between = before.get(close_paren..).unwrap_or("");
        let trimmed = between.trim();
        // If between `)` and the current position there is `:` and no `(`, it is a return type
        if trimmed.contains(':') && !between.contains('(') {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(PreferReadonlyParameterTypes, "test.ts");

    #[test]
    fn test_flags_mutable_array_param() {
        let source = "function sum(arr: number[]) { return 0; }";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "mutable array parameter should be flagged");
    }

    #[test]
    fn test_allows_readonly_array_param() {
        let source = "function sum(arr: readonly number[]) { return 0; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "readonly array parameter should not be flagged"
        );
    }

    #[test]
    fn test_flags_string_array_param() {
        let source = "function join(items: string[]) { return ''; }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "mutable string array parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_non_array_param() {
        let source = "function greet(name: string) { return name; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-array parameter should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_type_array() {
        let source = "function getItems(): string[] { return []; }";
        let diags = lint(source);
        assert!(diags.is_empty(), "array return type should not be flagged");
    }
}
