//! Rule: `jest/padding-around-test-blocks`
//!
//! Suggest adding blank lines before and after test blocks (`it`/`test`/`describe`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/padding-around-test-blocks";

/// Test block identifiers to check padding around.
const TEST_BLOCK_NAMES: &[&str] = &["it(", "test(", "describe("];

/// Suggests blank lines before and after test/describe blocks.
#[derive(Debug)]
pub struct PaddingAroundTestBlocks;

impl LintRule for PaddingAroundTestBlocks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require padding around test blocks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("it(")
            || source_text.contains("test(")
            || source_text.contains("describe("))
            && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = {
            let source = ctx.source_text();
            let lines: Vec<&str> = source.lines().collect();
            let mut violations: Vec<(String, Span)> = Vec::new();

            for (line_idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                for block_name in TEST_BLOCK_NAMES {
                    if !trimmed.starts_with(block_name) {
                        continue;
                    }

                    let line_start_in_source = lines.get(..line_idx).map_or(0, |prev| {
                        prev.iter()
                            .map(|l| l.len().saturating_add(1))
                            .sum::<usize>()
                    });
                    let trimmed_offset = line.len().saturating_sub(trimmed.len());
                    let abs_pos = line_start_in_source.saturating_add(trimmed_offset);

                    let is_word_boundary = abs_pos == 0
                        || source
                            .as_bytes()
                            .get(abs_pos.saturating_sub(1))
                            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

                    if !is_word_boundary {
                        continue;
                    }

                    if line_idx > 0 {
                        let prev_line =
                            lines.get(line_idx.saturating_sub(1)).copied().unwrap_or("");
                        let prev_trimmed = prev_line.trim();

                        if !prev_trimmed.is_empty()
                            && prev_trimmed != "{"
                            && !prev_trimmed.ends_with('{')
                            && !prev_trimmed.starts_with("//")
                            && !prev_trimmed.starts_with("/*")
                            && !TEST_BLOCK_NAMES.iter().any(|n| prev_trimmed.starts_with(n))
                        {
                            let start_u32 = u32::try_from(abs_pos).unwrap_or(0);
                            let end_u32 = start_u32.saturating_add(
                                u32::try_from(block_name.len().saturating_sub(1)).unwrap_or(0),
                            );
                            let msg = format!(
                                "Add a blank line before `{}`",
                                block_name
                                    .get(..block_name.len().saturating_sub(1))
                                    .unwrap_or(block_name),
                            );
                            violations.push((msg, Span::new(start_u32, end_u32)));
                        }
                    }
                }
            }

            violations
        };

        for (msg, span) in &violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: msg.to_owned(),
                span: *span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PaddingAroundTestBlocks)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_missing_padding_before_test() {
        let source = r"const x = 1;
test('works', () => { expect(x).toBe(1); });";
        let diags = lint(source);
        assert!(
            !diags.is_empty(),
            "missing blank line before `test` should be flagged"
        );
    }

    #[test]
    fn test_allows_padded_test() {
        let source = r"const x = 1;

test('works', () => { expect(x).toBe(1); });";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "test with blank line before should not be flagged"
        );
    }

    #[test]
    fn test_allows_first_test_in_describe() {
        let source = r"
describe('suite', () => {
    test('works', () => { expect(1).toBe(1); });
});";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "first test after describe opening brace should not be flagged"
        );
    }
}
