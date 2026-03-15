//! Rule: `typescript/no-unnecessary-type-parameters`
//!
//! Disallow type parameters that are only used once in a function or type
//! signature. A type parameter that appears in only one position provides no
//! type-safety benefit — it is equivalent to using the constraint type (or
//! `unknown`) directly. For example, `function f<T>(x: T): void` gains nothing
//! over `function f(x: unknown): void` because `T` is never reused.
//!
//! This is a simplified text-based check: it scans for generic type parameter
//! declarations and counts how many times each parameter name appears in the
//! surrounding signature text.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unnecessary-type-parameters";

/// Flags generic type parameters that appear only once in the signature,
/// providing no type-safety benefit.
#[derive(Debug)]
pub struct NoUnnecessaryTypeParameters;

impl LintRule for NoUnnecessaryTypeParameters {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow type parameters that are only used once".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();

        for finding in find_single_use_type_params(source) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Type parameter `{}` is used only once — consider removing it and using the type directly",
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

/// A type parameter that was found to be used only once.
struct SingleUseTypeParam {
    /// The type parameter name (e.g. `T`).
    name: String,
    /// Start byte offset in source.
    start: u32,
    /// End byte offset in source.
    end: u32,
}

/// Scan source text for generic type parameter declarations (between `<` and `>`)
/// and check if each parameter name appears only once in the surrounding
/// function/type signature.
fn find_single_use_type_params(source: &str) -> Vec<SingleUseTypeParam> {
    let mut results = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos: usize = 0;

    while pos < len {
        // Look for patterns like `function name<`, `=> <`, `type Name<`, or `class Name<`
        // by finding `<` that looks like the start of a type parameter list
        if bytes.get(pos).copied() != Some(b'<') {
            pos = pos.saturating_add(1);
            continue;
        }

        // Heuristic: the character before `<` should be an identifier char or `)`
        // (to distinguish from comparison operators)
        let before_ok = pos > 0
            && bytes
                .get(pos.saturating_sub(1))
                .is_none_or(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b')');

        if !before_ok {
            pos = pos.saturating_add(1);
            continue;
        }

        // Find matching `>`
        let angle_start = pos;
        let mut depth: usize = 1;
        let mut scan = pos.saturating_add(1);
        while scan < len && depth > 0 {
            match bytes.get(scan).copied() {
                Some(b'<') => depth = depth.saturating_add(1),
                Some(b'>') => depth = depth.saturating_sub(1),
                _ => {}
            }
            scan = scan.saturating_add(1);
        }

        if depth != 0 {
            pos = pos.saturating_add(1);
            continue;
        }

        let angle_end = scan; // one past the closing `>`
        let params_text = source
            .get(angle_start.saturating_add(1)..angle_end.saturating_sub(1))
            .unwrap_or("");

        // Extract type parameter names (split by `,`, take the first word before
        // `extends` or whitespace)
        let param_names = extract_type_param_names(params_text);

        if param_names.is_empty() {
            pos = angle_end;
            continue;
        }

        // Determine the signature region to count occurrences in.
        // Use the rest of the line (or until `{` / `;`) after the closing `>`.
        let sig_end = find_signature_end(source, angle_end);
        let signature_region = source.get(angle_end..sig_end).unwrap_or("");

        for param_name in &param_names {
            if param_name.is_empty() {
                continue;
            }

            // Count occurrences of the type parameter name as a whole word
            // in the signature region (after the type parameter list)
            let usage_count = count_word_occurrences(signature_region, param_name);

            // If the type parameter is used zero or one times in the rest of
            // the signature, it is unnecessary (the one in the declaration itself
            // is inside angle brackets, which we already excluded).
            if usage_count <= 1 {
                // Find the position of this parameter name in the angle brackets
                // for the span
                let param_offset = find_word_position(params_text, param_name);
                let abs_start = angle_start.saturating_add(1).saturating_add(param_offset);
                let abs_end = abs_start.saturating_add(param_name.len());

                let start_u32 = u32::try_from(abs_start).unwrap_or(0);
                let end_u32 = u32::try_from(abs_end).unwrap_or(start_u32);

                results.push(SingleUseTypeParam {
                    name: (*param_name).to_owned(),
                    start: start_u32,
                    end: end_u32,
                });
            }
        }

        pos = angle_end;
    }

    results
}

/// Extract type parameter names from the text between `<` and `>`.
///
/// Handles comma-separated params like `T, U extends Foo, V`.
fn extract_type_param_names(params_text: &str) -> Vec<&str> {
    params_text
        .split(',')
        .filter_map(|segment| {
            let trimmed = segment.trim();
            // Take everything before `extends`, `=`, or whitespace
            let name = trimmed
                .split_once(|c: char| c.is_whitespace())
                .map_or(trimmed, |(before, _)| before)
                .trim();
            if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

/// Find the end of the signature region (until `{`, `;`, or end of source).
fn find_signature_end(source: &str, start: usize) -> usize {
    let remaining = source.get(start..).unwrap_or("");
    for (i, ch) in remaining.char_indices() {
        if ch == '{' || ch == ';' {
            return start.saturating_add(i);
        }
    }
    source.len()
}

/// Count how many times `word` appears as a whole word in `text`.
fn count_word_occurrences(text: &str, word: &str) -> usize {
    let mut count: usize = 0;
    let mut search_from: usize = 0;
    let text_bytes = text.as_bytes();

    while let Some(pos) = text.get(search_from..).and_then(|s| s.find(word)) {
        let abs = search_from.saturating_add(pos);
        let after = abs.saturating_add(word.len());

        let before_ok = abs == 0
            || text_bytes
                .get(abs.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');
        let after_ok = text_bytes
            .get(after)
            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');

        if before_ok && after_ok {
            count = count.saturating_add(1);
        }
        search_from = after;
    }

    count
}

/// Find the byte offset of `word` as a whole word within `text`.
fn find_word_position(text: &str, word: &str) -> usize {
    let text_bytes = text.as_bytes();
    let mut search_from: usize = 0;

    while let Some(pos) = text.get(search_from..).and_then(|s| s.find(word)) {
        let abs = search_from.saturating_add(pos);
        let after = abs.saturating_add(word.len());

        let before_ok = abs == 0
            || text_bytes
                .get(abs.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');
        let after_ok = text_bytes
            .get(after)
            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');

        if before_ok && after_ok {
            return abs;
        }
        search_from = after;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoUnnecessaryTypeParameters, "test.ts");

    #[test]
    fn test_flags_single_use_type_param() {
        let diags = lint("function f<T>(x: T): void {}");
        assert_eq!(diags.len(), 1, "`T` used only once should be flagged");
    }

    #[test]
    fn test_allows_type_param_used_twice() {
        let diags = lint("function f<T>(x: T): T {}");
        assert!(
            diags.is_empty(),
            "`T` used in param and return should not be flagged"
        );
    }

    #[test]
    fn test_flags_unused_type_param() {
        let diags = lint("function f<T>(): void {}");
        assert_eq!(
            diags.len(),
            1,
            "`T` not used anywhere in signature should be flagged"
        );
    }

    #[test]
    fn test_allows_type_param_used_in_multiple_params() {
        let diags = lint("function f<T>(a: T, b: T): void {}");
        assert!(
            diags.is_empty(),
            "`T` used in multiple parameters should not be flagged"
        );
    }

    #[test]
    fn test_flags_one_of_multiple_params() {
        let diags = lint("function f<T, U>(a: T, b: T): U {}");
        // T is used twice (a: T, b: T) so it's fine
        // U is used once (return type) so it should be flagged
        assert_eq!(
            diags.len(),
            1,
            "only `U` should be flagged, `T` is used twice"
        );
    }
}
