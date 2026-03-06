//! Rule: `jest/consistent-test-it`
//!
//! Enforce using either `test` or `it` consistently. Flags when both `test(`
//! and `it(` are used in the same file.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/consistent-test-it";

/// Enforces consistent usage of `test` vs `it` in the same file.
#[derive(Debug)]
pub struct ConsistentTestIt;

impl NativeRule for ConsistentTestIt {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce consistent use of `test` or `it`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = {
            let source = ctx.source_text();

            let test_positions = find_call_positions(source, "test(");
            let it_positions = find_call_positions(source, "it(");

            if test_positions.is_empty() || it_positions.is_empty() {
                return;
            }

            let (minority_positions, minority_name, majority_name) =
                if test_positions.len() >= it_positions.len() {
                    (it_positions, "it", "test")
                } else {
                    (test_positions, "test", "it")
                };

            minority_positions
                .into_iter()
                .map(|pos| {
                    let start_u32 = u32::try_from(pos).unwrap_or(0);
                    let end_u32 =
                        start_u32.saturating_add(u32::try_from(minority_name.len()).unwrap_or(0));
                    let msg = format!(
                        "Prefer `{majority_name}` over `{minority_name}` — use a consistent test function name"
                    );
                    (msg, Span::new(start_u32, end_u32), majority_name.to_owned(), minority_name.to_owned())
                })
                .collect::<Vec<_>>()
        };

        for (msg, span, majority, minority) in &violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: msg.clone(),
                span: *span,
                severity: Severity::Warning,
                help: Some(format!("Replace `{minority}` with `{majority}`")),
                fix: Some(Fix {
                    message: format!("Replace with `{majority}`"),
                    edits: vec![Edit {
                        span: *span,
                        replacement: majority.clone(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Find all positions of a call pattern (e.g. `test(`) in source, ensuring word boundary.
fn find_call_positions(source: &str, pattern: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(pattern)) {
        let abs_pos = search_from.saturating_add(pos);

        let is_word_boundary = abs_pos == 0
            || source
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_none_or(|b| {
                    !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$' && *b != b'.'
                });

        if is_word_boundary {
            positions.push(abs_pos);
        }

        search_from = abs_pos.saturating_add(pattern.len());
    }

    positions
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentTestIt)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_mixed_test_and_it() {
        let source = r"
test('one', () => { expect(1).toBe(1); });
it('two', () => { expect(2).toBe(2); });
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "mixing `test` and `it` should flag the minority"
        );
    }

    #[test]
    fn test_allows_consistent_test() {
        let source = r"
test('one', () => { expect(1).toBe(1); });
test('two', () => { expect(2).toBe(2); });
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "consistent `test` usage should not be flagged"
        );
    }

    #[test]
    fn test_allows_consistent_it() {
        let source = r"
it('one', () => { expect(1).toBe(1); });
it('two', () => { expect(2).toBe(2); });
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "consistent `it` usage should not be flagged"
        );
    }
}
