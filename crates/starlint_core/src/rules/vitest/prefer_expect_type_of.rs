//! Rule: `vitest/prefer-expect-type-of`
//!
//! Suggest using `expectTypeOf` for type assertions instead of relying on
//! `@ts-expect-error` comments. Vitest provides `expectTypeOf` as a proper
//! type-testing utility that gives better error messages and is more
//! maintainable than `@ts-expect-error`-based type tests.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-expect-type-of";

/// Suggest `expectTypeOf` over `@ts-expect-error` for type testing.
#[derive(Debug)]
pub struct PreferExpectTypeOf;

impl NativeRule for PreferExpectTypeOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `expectTypeOf` for type assertions over `@ts-expect-error`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = find_ts_expect_error_in_comments(ctx.source_text());

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Prefer `expectTypeOf()` for type assertions instead of `@ts-expect-error`"
                        .to_owned(),
                span,
                severity: Severity::Warning,
                help: Some("Use `expectTypeOf()` for type-level assertions".to_owned()),
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Find `@ts-expect-error` occurrences inside comments.
fn find_ts_expect_error_in_comments(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let needle = "@ts-expect-error";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(pos);

        let before = source.get(..abs_pos).unwrap_or("");
        let line_start = before.rfind('\n').map_or(0, |p| p.saturating_add(1));
        let line_prefix = before.get(line_start..).unwrap_or("").trim();

        if line_prefix.starts_with("//")
            || line_prefix.starts_with("/*")
            || line_prefix.starts_with('*')
        {
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = u32::try_from(abs_pos.saturating_add(needle.len())).unwrap_or(start);
            results.push(Span::new(start, end));
        }

        search_start = abs_pos.saturating_add(1);
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferExpectTypeOf)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_ts_expect_error_comment() {
        let source = "// @ts-expect-error\nconst x: number = 'hello';";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`@ts-expect-error` in comment should be flagged"
        );
    }

    #[test]
    fn test_allows_expect_type_of() {
        let source = "expectTypeOf(fn).toBeFunction();";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`expectTypeOf` usage should not be flagged"
        );
    }

    #[test]
    fn test_allows_code_without_ts_expect_error() {
        let source = "const x: number = 1;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "code without `@ts-expect-error` should not be flagged"
        );
    }
}
