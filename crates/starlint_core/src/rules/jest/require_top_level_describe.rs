//! Rule: `jest/require-top-level-describe`
//!
//! Warn when `it`/`test` are used at the top level without a `describe` wrapper.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/require-top-level-describe";

/// Flags `it` and `test` calls that are not inside any `describe` block.
#[derive(Debug)]
pub struct RequireTopLevelDescribe;

impl NativeRule for RequireTopLevelDescribe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `test`/`it` to be inside a `describe` block".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            let test_patterns = ["it(", "test("];
            let mut violations: Vec<(String, Span)> = Vec::new();

            for pattern in &test_patterns {
                let mut search_start: usize = 0;

                while let Some(pos) = source.get(search_start..).and_then(|s| s.find(pattern)) {
                    let abs_pos = search_start.saturating_add(pos);

                    let is_word_boundary = abs_pos == 0
                        || source
                            .as_bytes()
                            .get(abs_pos.saturating_sub(1))
                            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

                    if is_word_boundary && !is_inside_describe(source, abs_pos) {
                        let name_len = pattern.len().saturating_sub(1);
                        let start_u32 = u32::try_from(abs_pos).unwrap_or(0);
                        let end_u32 =
                            start_u32.saturating_add(u32::try_from(name_len).unwrap_or(0));
                        let msg = format!(
                            "`{}` should be inside a `describe` block",
                            pattern.get(..name_len).unwrap_or(pattern),
                        );
                        violations.push((msg, Span::new(start_u32, end_u32)));
                    }

                    search_start = abs_pos.saturating_add(pattern.len());
                }
            }

            violations
        };

        for (msg, span) in &violations {
            ctx.report_warning(RULE_NAME, msg, *span);
        }
    }
}

/// Check if a position is inside a `describe` block by counting brace nesting.
fn is_inside_describe(source: &str, pos: usize) -> bool {
    let before = source.get(..pos).unwrap_or("");
    let needle = "describe(";

    let mut search_from: usize = 0;

    while let Some(desc_pos) = before.get(search_from..).and_then(|s| s.find(needle)) {
        let abs_desc_pos = search_from.saturating_add(desc_pos);

        // Ensure word boundary
        let is_word_boundary = abs_desc_pos == 0
            || before
                .as_bytes()
                .get(abs_desc_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

        if is_word_boundary {
            let after_desc = abs_desc_pos.saturating_add(needle.len());

            // Find the opening brace of the describe callback
            if let Some(brace_offset) = source.get(after_desc..pos).and_then(|s| s.find('{')) {
                let brace_pos = after_desc.saturating_add(brace_offset);

                if brace_pos < pos {
                    let between = source.get(brace_pos..pos).unwrap_or("");
                    let open_count = between.chars().filter(|c| *c == '{').count();
                    let close_count = between.chars().filter(|c| *c == '}').count();

                    if open_count > close_count {
                        return true;
                    }
                }
            }
        }

        search_from = abs_desc_pos.saturating_add(needle.len());
    }

    false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireTopLevelDescribe)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_top_level_test() {
        let source = "test('works', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "top-level `test` should be flagged");
    }

    #[test]
    fn test_flags_top_level_it() {
        let source = "it('works', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "top-level `it` should be flagged");
    }

    #[test]
    fn test_allows_test_inside_describe() {
        let source = r"
describe('suite', () => {
    test('works', () => { expect(1).toBe(1); });
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`test` inside `describe` should not be flagged"
        );
    }
}
