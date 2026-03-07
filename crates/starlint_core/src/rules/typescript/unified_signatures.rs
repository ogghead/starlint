//! Rule: `typescript/unified-signatures`
//!
//! Require that function overload signatures be merged into a single signature
//! using union types when possible. Two consecutive overloads that differ only
//! in one parameter type can be combined into a single signature with a union
//! type for that parameter, reducing redundancy and improving readability.
//!
//! This is a simplified text-based check: it scans for consecutive function
//! declarations with the same name and flags them when they appear to be
//! overloads that could be merged.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/unified-signatures";

/// Flags consecutive function overload signatures that could be merged into a
/// single signature using union types.
#[derive(Debug)]
pub struct UnifiedSignatures;

impl LintRule for UnifiedSignatures {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Require function overloads to be merged when they differ in only one parameter type"
                    .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();

        for finding in find_mergeable_overloads(source) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Function `{}` has overload signatures that differ only in one parameter type — merge them using a union type",
                    finding.name
                ),
                span: Span::new(finding.start, finding.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// A pair of overloads that could be merged.
struct MergeableOverload {
    /// The function name.
    name: String,
    /// Start byte offset of the second overload in source.
    start: u32,
    /// End byte offset of the second overload in source.
    end: u32,
}

/// Information about a function overload signature extracted from source text.
struct OverloadInfo {
    /// The function name.
    name: String,
    /// The parameter strings (e.g. `["x: string", "y: number"]`).
    params: Vec<String>,
    /// Start byte offset of the line in source.
    line_start: usize,
    /// End byte offset of the line in source.
    line_end: usize,
}

/// Scan source text for consecutive function overload declarations that could
/// be merged into a single signature.
fn find_mergeable_overloads(source: &str) -> Vec<MergeableOverload> {
    let mut results = Vec::new();
    let overloads = extract_overload_signatures(source);

    // Group consecutive overloads by name
    let mut i: usize = 0;
    while i < overloads.len() {
        let Some(current) = overloads.get(i) else {
            break;
        };
        let current_name = &current.name;
        let group_start = i;

        // Collect consecutive overloads with the same name
        while overloads.get(i).is_some_and(|o| o.name == *current_name) {
            i = i.saturating_add(1);
        }

        let group_end = i;
        if group_end.saturating_sub(group_start) < 2 {
            continue;
        }

        // Check pairs of overloads in the group
        let mut j = group_start;
        while j.saturating_add(1) < group_end {
            let next = j.saturating_add(1);
            if let (Some(a), Some(b)) = (overloads.get(j), overloads.get(next)) {
                if can_merge_overloads(a, b) {
                    let start_u32 = u32::try_from(b.line_start).unwrap_or(0);
                    let end_u32 = u32::try_from(b.line_end).unwrap_or(start_u32);

                    results.push(MergeableOverload {
                        name: current_name.clone(),
                        start: start_u32,
                        end: end_u32,
                    });
                }
            }

            j = j.saturating_add(1);
        }
    }

    results
}

/// Extract function overload signatures from source text.
///
/// Looks for lines matching `function name(params): type;` (TypeScript overload
/// declarations end with `;`, not `{`).
fn extract_overload_signatures(source: &str) -> Vec<OverloadInfo> {
    let mut overloads = Vec::new();
    let mut offset: usize = 0;

    for line in source.lines() {
        let trimmed = line.trim();
        let line_start = offset;
        let line_end = offset.saturating_add(line.len());

        // Match "function name(" pattern — must end with ";" (overload, not implementation)
        if trimmed.ends_with(';') {
            if let Some(info) = parse_overload_line(trimmed, line_start, line_end) {
                overloads.push(info);
            }
        }

        // +1 for the newline character
        offset = line_end.saturating_add(1);
    }

    overloads
}

/// Try to parse a single line as a function overload signature.
fn parse_overload_line(line: &str, line_start: usize, line_end: usize) -> Option<OverloadInfo> {
    // Strip "export " prefix if present
    let stripped = line.strip_prefix("export ").unwrap_or(line);
    // Must start with "function "
    let after_fn = stripped.strip_prefix("function ")?;

    // Extract function name (up to first `(` or `<`)
    let name_end = after_fn.find(['(', '<'])?;
    let name = after_fn.get(..name_end)?.trim();

    if name.is_empty() {
        return None;
    }

    // Extract parameters between `(` and `)`
    let paren_start = after_fn.find('(')?;
    let paren_end = find_matching_paren(after_fn, paren_start)?;
    let params_text = after_fn.get(paren_start.saturating_add(1)..paren_end)?;

    let params = split_params(params_text);

    Some(OverloadInfo {
        name: name.to_owned(),
        params,
        line_start,
        line_end,
    })
}

/// Find the matching closing `)` for an opening `(` at `start`.
fn find_matching_paren(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: usize = 0;
    let mut pos = start;

    while pos < bytes.len() {
        match bytes.get(pos).copied() {
            Some(b'(') => depth = depth.saturating_add(1),
            Some(b')') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
        pos = pos.saturating_add(1);
    }

    None
}

/// Split parameter text by commas, respecting nested angle brackets and parens.
fn split_params(params_text: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut current = String::new();
    let mut depth: usize = 0;

    for ch in params_text.chars() {
        match ch {
            '<' | '(' => {
                depth = depth.saturating_add(1);
                current.push(ch);
            }
            '>' | ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_owned();
                if !trimmed.is_empty() {
                    params.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let trimmed = current.trim().to_owned();
    if !trimmed.is_empty() {
        params.push(trimmed);
    }

    params
}

/// Check if two overloads differ in exactly one parameter type.
///
/// Two overloads can be merged if they have the same number of parameters
/// and differ in the type annotation of exactly one parameter.
fn can_merge_overloads(a: &OverloadInfo, b: &OverloadInfo) -> bool {
    if a.params.len() != b.params.len() {
        return false;
    }

    let mut diff_count: usize = 0;

    for (pa, pb) in a.params.iter().zip(b.params.iter()) {
        // Extract the type portion (after the `:`)
        let type_a = extract_param_type(pa);
        let type_b = extract_param_type(pb);

        if type_a != type_b {
            diff_count = diff_count.saturating_add(1);
            if diff_count > 1 {
                return false;
            }
        }
    }

    diff_count == 1
}

/// Extract the type annotation from a parameter string.
///
/// For `x: string`, returns `string`. For `x?: number`, returns `number`.
/// If no `:` is found, returns the whole string.
fn extract_param_type(param: &str) -> &str {
    match param.split_once(':') {
        Some((_, ty)) => ty.trim(),
        None => param.trim(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(UnifiedSignatures)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_overloads_differing_in_one_param_type() {
        let source = "function f(x: string): void;\nfunction f(x: number): void;\nfunction f(x: string | number): void {}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "overloads differing in one param type should be flagged"
        );
    }

    #[test]
    fn test_allows_overloads_differing_in_multiple_params() {
        let source = "function f(x: string, y: number): void;\nfunction f(x: number, y: string): void;\nfunction f(x: any, y: any): void {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "overloads differing in multiple params should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_param_count() {
        let source = "function f(x: string): void;\nfunction f(x: string, y: number): void;\nfunction f(x: string, y?: number): void {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "overloads with different param counts should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_function() {
        let source = "function f(x: string): void {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "a single function without overloads should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_function_names() {
        let source = "function f(x: string): void;\nfunction g(x: number): void;\nfunction f(x: any): void {}\nfunction g(x: any): void {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "different function names should not be grouped as overloads"
        );
    }
}
