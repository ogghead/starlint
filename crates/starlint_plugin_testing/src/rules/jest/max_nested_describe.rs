//! Rule: `jest/max-nested-describe`
//!
//! Warn when `describe` blocks are nested too deeply (default: > 5).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/max-nested-describe";

/// Default maximum nesting depth for `describe` blocks.
const DEFAULT_MAX_DEPTH: usize = 5;

/// Flags `describe` blocks that are nested too deeply.
#[derive(Debug)]
pub struct MaxNestedDescribe;

impl LintRule for MaxNestedDescribe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Limit the nesting depth of `describe` blocks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("describe(") && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            find_deeply_nested_describes(source)
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

/// Single forward pass: track brace depth and describe nesting with a stack.
///
/// Previous implementation was O(m²) — for each `describe(` it re-scanned all
/// previous `describe(` blocks and counted braces between them.
/// This implementation is O(n) — single scan through the source.
fn find_deeply_nested_describes(source: &str) -> Vec<(usize, Span)> {
    let needle = "describe(";
    let mut violations: Vec<(usize, Span)> = Vec::new();

    // Stack of brace_depth values when each describe was encountered.
    let mut describe_stack: Vec<usize> = Vec::new();
    let mut brace_depth: usize = 0;
    let mut search_start: usize = 0;

    // Use find-based scanning — avoids manual byte indexing.
    // Process braces and keywords between each `describe(` match.
    // First, collect all describe positions.
    let mut describe_positions: Vec<usize> = Vec::new();
    while let Some(offset) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(offset);
        let is_word_boundary = abs_pos == 0
            || source
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');
        if is_word_boundary {
            describe_positions.push(abs_pos);
        }
        search_start = abs_pos.saturating_add(needle.len());
    }

    // Now do a single forward pass counting braces, checking depth at each describe.
    // Use byte iteration since `{`, `}` are ASCII — avoids UTF-8 decoding overhead.
    let bytes = source.as_bytes();
    let mut desc_idx: usize = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'{' {
            brace_depth = brace_depth.saturating_add(1);
        } else if b == b'}' {
            brace_depth = brace_depth.saturating_sub(1);
            while describe_stack.last().is_some_and(|&d| d == brace_depth) {
                describe_stack.pop();
            }
        }
        // Check if we've reached the next describe position
        if desc_idx < describe_positions.len()
            && describe_positions.get(desc_idx).copied() == Some(i)
        {
            let depth = describe_stack.len().saturating_add(1);
            if depth > DEFAULT_MAX_DEPTH {
                let start_u32 = u32::try_from(i).unwrap_or(0);
                let end_u32 = u32::try_from(i.saturating_add(needle.len())).unwrap_or(start_u32);
                violations.push((depth, Span::new(start_u32, end_u32)));
            }
            describe_stack.push(brace_depth);
            desc_idx = desc_idx.saturating_add(1);
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(MaxNestedDescribe);

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
