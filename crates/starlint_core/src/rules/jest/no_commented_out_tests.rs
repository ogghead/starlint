//! Rule: `jest/no-commented-out-tests`
//!
//! Warn when test code is commented out (e.g. `// it(`, `// test(`, `// describe(`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for NoCommentedOutTests {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow commented-out tests".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = find_commented_tests(ctx.source_text());

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Unexpected commented-out test — remove or uncomment".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCommentedOutTests)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
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
