//! Rule: `jest/max-nested-describe`
//!
//! Warn when `describe` blocks are nested too deeply (default: > 5).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/max-nested-describe";

/// Default maximum nesting depth for `describe` blocks.
const DEFAULT_MAX_DEPTH: usize = 5;

/// Flags `describe` blocks that are nested too deeply.
#[derive(Debug)]
pub struct MaxNestedDescribe;

impl NativeRule for MaxNestedDescribe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Limit the nesting depth of `describe` blocks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            let needle = "describe(";
            let mut violations: Vec<(usize, Span)> = Vec::new();

            let mut search_start: usize = 0;

            while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
                let abs_pos = search_start.saturating_add(pos);

                let is_word_boundary = abs_pos == 0
                    || source
                        .as_bytes()
                        .get(abs_pos.saturating_sub(1))
                        .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

                if is_word_boundary {
                    let depth = count_describe_nesting(source, abs_pos);
                    if depth > DEFAULT_MAX_DEPTH {
                        let end = abs_pos.saturating_add(needle.len());
                        let start_u32 = u32::try_from(abs_pos).unwrap_or(0);
                        let end_u32 = u32::try_from(end).unwrap_or(start_u32);
                        violations.push((depth, Span::new(start_u32, end_u32)));
                    }
                }

                search_start = abs_pos.saturating_add(needle.len());
            }

            violations
        };

        for (depth, span) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`describe` is nested {depth} levels deep (max: {DEFAULT_MAX_DEPTH})"
                ),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Count how deeply a `describe` call at `pos` is nested inside other `describe` blocks.
fn count_describe_nesting(source: &str, pos: usize) -> usize {
    let before = source.get(..pos).unwrap_or("");
    let needle = "describe(";

    // For each `describe(` before our position, check if our position
    // falls inside its brace-delimited body.
    let mut depth: usize = 0;
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
            if let Some(brace_offset) = source.get(after_desc..).and_then(|s| s.find('{')) {
                let brace_pos = after_desc.saturating_add(brace_offset);

                // Only count if our position is after this opening brace
                if brace_pos < pos {
                    // Check if our position is still inside this describe's body
                    let between = source.get(brace_pos..pos).unwrap_or("");
                    let open_count = between.chars().filter(|c| *c == '{').count();
                    let close_count = between.chars().filter(|c| *c == '}').count();

                    if open_count > close_count {
                        depth = depth.saturating_add(1);
                    }
                }
            }
        }

        search_from = abs_desc_pos.saturating_add(needle.len());
    }

    // Add 1 for the describe at `pos` itself
    depth.saturating_add(1)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxNestedDescribe)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_deeply_nested_describe() {
        let source = r"
describe('1', () => {
  describe('2', () => {
    describe('3', () => {
      describe('4', () => {
        describe('5', () => {
          describe('6', () => {
            test('deep', () => { expect(1).toBe(1); });
          });
        });
      });
    });
  });
});
";
        let diags = lint(source);
        assert!(
            !diags.is_empty(),
            "deeply nested describe (6 levels) should be flagged"
        );
    }

    #[test]
    fn test_allows_shallow_nesting() {
        let source = r"
describe('1', () => {
  describe('2', () => {
    test('shallow', () => { expect(1).toBe(1); });
  });
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "shallow nesting (2 levels) should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_describe() {
        let source = r"
describe('single', () => {
  test('ok', () => { expect(1).toBe(1); });
});
";
        let diags = lint(source);
        assert!(diags.is_empty(), "single describe should not be flagged");
    }
}
