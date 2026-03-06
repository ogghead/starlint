//! Rule: `typescript/use-unknown-in-catch-callback-variable`
//!
//! Enforce `unknown` type for catch callback variables. In
//! `promise.catch((err) => ...)`, the `err` parameter should be typed as
//! `unknown` rather than `any`, `Error`, or other concrete types. Using
//! `unknown` forces explicit type narrowing, which is safer.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.catch()` callbacks whose parameter is typed as something other
/// than `unknown`.
#[derive(Debug)]
pub struct UseUnknownInCatchCallbackVariable;

impl NativeRule for UseUnknownInCatchCallbackVariable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/use-unknown-in-catch-callback-variable".to_owned(),
            description: "Enforce `unknown` type annotation on catch callback parameters"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings = find_bad_catch_params(ctx.source_text());

        for (type_name, start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/use-unknown-in-catch-callback-variable".to_owned(),
                message: format!(
                    "Catch callback parameter should be typed as `unknown` instead of `{type_name}`"
                ),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Scan source text for `.catch((<param>: <type>)` patterns where the type
/// annotation is not `unknown`.
///
/// Returns a list of `(type_name, start_offset, end_offset)` tuples for each
/// occurrence that should be flagged.
fn find_bad_catch_params(source: &str) -> Vec<(String, u32, u32)> {
    let mut results = Vec::new();
    let catch_pattern = ".catch(";
    let catch_len = catch_pattern.len();

    let mut search_from: usize = 0;
    while let Some(pos) = source
        .get(search_from..)
        .and_then(|s| s.find(catch_pattern))
    {
        let absolute_pos = search_from.saturating_add(pos);
        let after_catch = absolute_pos.saturating_add(catch_len);

        if let Some((type_name, type_start, type_end)) = extract_typed_param(source, after_catch) {
            if type_name != "unknown" {
                let start = u32::try_from(type_start).unwrap_or(0);
                let end = u32::try_from(type_end).unwrap_or(start);
                results.push((type_name, start, end));
            }
        }

        search_from = after_catch;
    }

    results
}

/// Look for a typed parameter pattern `(<name>: <type>)` starting at the
/// given position.
///
/// Returns `Some((type_name, type_start_offset, type_end_offset))` if a typed
/// parameter is found with a type annotation, or `None` otherwise.
fn extract_typed_param(source: &str, start: usize) -> Option<(String, usize, usize)> {
    let rest = source.get(start..).unwrap_or("");

    // Allow optional opening paren for arrow functions: `(err: Type) =>`
    let after_paren = if rest.starts_with('(') {
        rest.get(1..)?.trim_start()
    } else {
        rest.trim_start()
    };

    // Find the colon that separates param name from type annotation
    let colon_pos = after_paren.find(':')?;

    // Validate that everything before the colon looks like a parameter name
    let param_name = after_paren.get(..colon_pos)?.trim();
    if param_name.is_empty()
        || !param_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
    {
        return None;
    }

    // Extract the type annotation after the colon
    let after_colon = after_paren.get(colon_pos.saturating_add(1)..)?.trim_start();

    // The type extends until `)` or `=>`
    let type_end_pos = after_colon.find(')').or_else(|| after_colon.find("=>"))?;
    let type_name = after_colon.get(..type_end_pos)?.trim();

    if type_name.is_empty() {
        return None;
    }

    // Calculate absolute positions of the type annotation by searching within
    // the source string from the colon position forward, avoiding pointer casts.
    let colon_abs = start
        .saturating_add(rest.len().saturating_sub(after_paren.len()))
        .saturating_add(colon_pos)
        .saturating_add(1);
    let after_colon_in_source = source.get(colon_abs..).unwrap_or("");
    let type_start_in_after_colon = after_colon_in_source.find(type_name).unwrap_or(0);
    let type_abs_start = colon_abs.saturating_add(type_start_in_after_colon);
    let type_abs_end = type_abs_start.saturating_add(type_name.len());

    Some((type_name.to_owned(), type_abs_start, type_abs_end))
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(UseUnknownInCatchCallbackVariable)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_catch_with_any_type() {
        let diags = lint("promise.catch((err: any) => console.log(err));");
        assert_eq!(
            diags.len(),
            1,
            "`.catch()` param typed as `any` should be flagged"
        );
    }

    #[test]
    fn test_flags_catch_with_error_type() {
        let diags = lint("promise.catch((err: Error) => console.log(err));");
        assert_eq!(
            diags.len(),
            1,
            "`.catch()` param typed as `Error` should be flagged"
        );
    }

    #[test]
    fn test_allows_catch_with_unknown_type() {
        let diags = lint("promise.catch((err: unknown) => console.log(err));");
        assert!(
            diags.is_empty(),
            "`.catch()` param typed as `unknown` should not be flagged"
        );
    }

    #[test]
    fn test_allows_catch_without_type_annotation() {
        let diags = lint("promise.catch((err) => console.log(err));");
        assert!(
            diags.is_empty(),
            "`.catch()` param without type annotation should not be flagged"
        );
    }

    #[test]
    fn test_ignores_non_catch_method() {
        let diags = lint("promise.then((result: string) => console.log(result));");
        assert!(
            diags.is_empty(),
            "non-catch method calls should not be flagged"
        );
    }
}
