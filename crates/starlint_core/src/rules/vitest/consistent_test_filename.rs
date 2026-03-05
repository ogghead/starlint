//! Rule: `vitest/consistent-test-filename`
//!
//! Warn when a test file does not match the expected naming convention.
//! Test files should include `.test.` or `.spec.` in their filename.
//! This rule runs once per file and inspects the file path.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/consistent-test-filename";

/// Expected patterns in test file names.
const TEST_PATTERNS: &[&str] = &[".test.", ".spec."];

/// Warn when test file doesn't match `.test.` or `.spec.` naming convention.
#[derive(Debug)]
pub struct ConsistentTestFilename;

impl NativeRule for ConsistentTestFilename {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce consistent test file naming convention (`.test.` or `.spec.`)"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let file_path = ctx.file_path();

        // Only check files that look like they belong in a test directory
        // or contain test-like content. We inspect the filename portion.
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.is_empty() {
            return;
        }

        // Only apply to JS/TS files.
        let extension = std::path::Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let is_js_ts = matches!(
            extension.to_ascii_lowercase().as_str(),
            "js" | "ts" | "jsx" | "tsx" | "mjs" | "mts"
        );

        if !is_js_ts {
            return;
        }

        // Check if the source contains test-like calls (it/test/describe).
        // Only flag files that actually contain test code.
        let source = ctx.source_text();
        let has_test_code =
            source.contains("test(") || source.contains("it(") || source.contains("describe(");

        if !has_test_code {
            return;
        }

        // Check if the filename matches the convention.
        let matches_convention = TEST_PATTERNS.iter().any(|p| file_name.contains(p));

        if !matches_convention {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Test file should include `.test.` or `.spec.` in its filename".to_owned(),
                span: Span::new(0, 0),
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentTestFilename)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_test_code_without_convention() {
        let source = r#"test("works", () => { expect(1).toBe(1); });"#;
        let diags = lint_with_path(source, Path::new("utils.ts"));
        assert_eq!(
            diags.len(),
            1,
            "test code in file without .test. or .spec. should be flagged"
        );
    }

    #[test]
    fn test_allows_test_file_convention() {
        let source = r#"test("works", () => { expect(1).toBe(1); });"#;
        let diags = lint_with_path(source, Path::new("utils.test.ts"));
        assert!(
            diags.is_empty(),
            "file with .test. in name should not be flagged"
        );
    }

    #[test]
    fn test_allows_spec_file_convention() {
        let source = r#"describe("utils", () => { it("works", () => {}); });"#;
        let diags = lint_with_path(source, Path::new("utils.spec.ts"));
        assert!(
            diags.is_empty(),
            "file with .spec. in name should not be flagged"
        );
    }
}
