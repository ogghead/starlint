//! Rule: `typescript/non-nullable-type-assertion-style`
//!
//! Prefer non-null assertions (`x!`) over explicit `as NonNullable<...>` type
//! assertions when removing `null` or `undefined` from a type. The non-null
//! assertion operator is more concise and idiomatic in TypeScript.
//!
//! Since full type-checking is unavailable, this rule flags `as NonNullable<`
//! patterns in source text as a proxy for the broader class of assertions that
//! could be replaced with `!`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `as NonNullable<...>` assertions that could use `!` instead.
#[derive(Debug)]
pub struct NonNullableTypeAssertionStyle;

impl NativeRule for NonNullableTypeAssertionStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/non-nullable-type-assertion-style".to_owned(),
            description: "Prefer `!` non-null assertion over `as NonNullable<...>`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    #[allow(clippy::as_conversions)]
    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();
        let findings = find_non_nullable_assertions(&source);

        for (start, end) in findings {
            let fix = source
                .get(start as usize..end as usize)
                .and_then(|as_text| {
                    // Extract the type inside NonNullable<...>
                    let inner_start = "as NonNullable<".len();
                    let inner = as_text.get(inner_start..as_text.len().saturating_sub(1))?;
                    (!inner.is_empty()).then(|| Fix {
                        message: "Use non-null assertion (`!`) instead".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(start, end),
                            replacement: format!("as {inner}!"),
                        }],
                    })
                });

            ctx.report(Diagnostic {
                rule_name: "typescript/non-nullable-type-assertion-style".to_owned(),
                message: "Use non-null assertion (`!`) instead of `as NonNullable<...>`".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Replace with non-null assertion operator".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Scan source text for `as NonNullable<` patterns that suggest a non-null
/// assertion could be used instead.
///
/// Returns a list of `(start_offset, end_offset)` tuples for each occurrence.
fn find_non_nullable_assertions(source: &str) -> Vec<(u32, u32)> {
    const PATTERN: &str = "as NonNullable<";

    let mut results = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(PATTERN)) {
        let absolute_pos = search_from.saturating_add(pos);
        let pattern_end = absolute_pos.saturating_add(PATTERN.len());

        // Find the matching closing `>` to get the full span.
        // Track nesting depth for generics like `NonNullable<Map<K, V>>`.
        let end = find_matching_angle_bracket(source, pattern_end).unwrap_or(pattern_end);

        let start = u32::try_from(absolute_pos).unwrap_or(0);
        let end_u32 = u32::try_from(end).unwrap_or(start);
        results.push((start, end_u32));

        search_from = pattern_end;
    }

    results
}

/// Starting after the opening `<`, find the position just past the matching `>`.
///
/// Returns `None` if no matching bracket is found.
fn find_matching_angle_bracket(source: &str, start: usize) -> Option<usize> {
    let mut depth: u32 = 1;
    let remainder = source.get(start..)?;

    for (i, ch) in remainder.char_indices() {
        match ch {
            '<' => {
                depth = depth.saturating_add(1);
            }
            '>' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    // Return position just past the closing `>`
                    return Some(start.saturating_add(i).saturating_add(1));
                }
            }
            _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NonNullableTypeAssertionStyle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_as_non_nullable() {
        let diags = lint("const x = value as NonNullable<string | null>;");
        assert_eq!(diags.len(), 1, "`as NonNullable<...>` should be flagged");
    }

    #[test]
    fn test_flags_nested_generic() {
        let diags = lint("const x = value as NonNullable<Map<string, number>>;");
        assert_eq!(
            diags.len(),
            1,
            "`as NonNullable<...>` with nested generics should be flagged"
        );
    }

    #[test]
    fn test_allows_non_null_assertion() {
        let diags = lint("const x = value!;");
        assert!(
            diags.is_empty(),
            "non-null assertion operator should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_as_assertion() {
        let diags = lint("const x = value as string;");
        assert!(
            diags.is_empty(),
            "regular type assertion should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_occurrences() {
        let diags = lint("const a = x as NonNullable<T>;\nconst b = y as NonNullable<U>;");
        assert_eq!(
            diags.len(),
            2,
            "both `as NonNullable<...>` assertions should be flagged"
        );
    }
}
