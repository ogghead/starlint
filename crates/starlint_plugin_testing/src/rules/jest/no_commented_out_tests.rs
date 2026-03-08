//! Rule: `jest/no-commented-out-tests`
//!
//! Warn when test code is commented out (e.g. `// it(`, `// test(`, `// describe(`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-commented-out-tests";

/// Patterns that indicate commented-out test code.
const COMMENT_PATTERNS: &[&str] = &[
    "// it(",
    "// it.(",
    "// test(",
    "// test.(",
    "// describe(",
    "// describe.(",
    "// xit(",
    "// xtest(",
    "// xdescribe(",
    "// fit(",
    "// fdescribe(",
    "/* it(",
    "/* test(",
    "/* describe(",
];

/// Flags commented-out test code.
#[derive(Debug)]
pub struct NoCommentedOutTests;

impl LintRule for NoCommentedOutTests {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow commented-out tests".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let diagnostics = {
            let source = ctx.source_text();
            let violations = find_commented_tests(source);
            violations
                .into_iter()
                .map(|span| {
                    // Delete the commented-out line (including trailing newline if present)
                    let delete_end = source
                        .as_bytes()
                        .get(usize::try_from(span.end).unwrap_or(0))
                        .copied()
                        .and_then(|b| (b == b'\n').then(|| span.end.saturating_add(1)))
                        .unwrap_or(span.end);
                    Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Unexpected commented-out test — remove or uncomment".to_owned(),
                        span,
                        severity: Severity::Warning,
                        help: None,
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Remove commented-out test".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(span.start, delete_end),
                                replacement: String::new(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    }
                })
                .collect::<Vec<_>>()
        };

        for diag in diagnostics {
            ctx.report(diag);
        }
    }
}

/// Scan source text for commented-out test patterns and return their spans.
fn find_commented_tests(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let mut byte_offset: u32 = 0;

    for line in source.lines() {
        let line_len = u32::try_from(line.len()).unwrap_or(0);
        let trimmed = line.trim();

        for pattern in COMMENT_PATTERNS {
            if trimmed.starts_with(pattern) {
                let offset_in_line =
                    u32::try_from(line.len().saturating_sub(trimmed.len())).unwrap_or(0);
                let start = byte_offset.saturating_add(offset_in_line);
                let end = byte_offset.saturating_add(line_len);
                results.push(Span::new(start, end));
                break;
            }
        }

        // +1 for the newline character
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoCommentedOutTests)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_commented_it() {
        let diags = lint("// it('should work', () => {});");
        assert_eq!(diags.len(), 1, "commented-out `it` should be flagged");
    }

    #[test]
    fn test_flags_commented_test() {
        let diags = lint("// test('should work', () => {});");
        assert_eq!(diags.len(), 1, "commented-out `test` should be flagged");
    }

    #[test]
    fn test_allows_regular_comments() {
        let diags = lint("// This is a regular comment");
        assert!(diags.is_empty(), "regular comments should not be flagged");
    }
}
