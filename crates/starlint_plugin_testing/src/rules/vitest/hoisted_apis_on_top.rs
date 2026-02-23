//! Rule: `vitest/hoisted-apis-on-top`
//!
//! Warn when `vi.hoisted()` calls are not at the top of the file.
//! `vi.hoisted()` is designed to be hoisted to the top of the module by
//! Vitest's transform, but for readability and clarity it should also be
//! placed at the top of the source file, before any other statements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/hoisted-apis-on-top";

/// Warn when `vi.hoisted()` is not at the top of the file.
#[derive(Debug)]
pub struct HoistedApisOnTop;

impl LintRule for HoistedApisOnTop {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `vi.hoisted()` calls at the top of the file".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("vi.hoisted") && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = find_misplaced_hoisted(ctx.source_text());

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`vi.hoisted()` should be at the top of the file, before other statements"
                    .to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Find `vi.hoisted()` calls that are not at the top of the file.
fn find_misplaced_hoisted(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let needle = "vi.hoisted(";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(pos);

        // Check if there are any non-import, non-comment, non-blank
        // statements before this `vi.hoisted()` call.
        let before = source.get(..abs_pos).unwrap_or("");

        let has_non_hoisted_code = before.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with("import ")
                && !trimmed.starts_with("import{")
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
                && !trimmed.starts_with('*')
                && !trimmed.starts_with("*/")
                && !trimmed.starts_with("vi.hoisted(")
                && !trimmed.starts_with("const ")
                && !trimmed.starts_with("export ")
        });

        if has_non_hoisted_code {
            let end_pos = abs_pos.saturating_add(needle.len());
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = u32::try_from(end_pos).unwrap_or(start);
            results.push(Span::new(start, end));
        }

        search_start = abs_pos.saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(HoistedApisOnTop)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_hoisted_after_code() {
        let source = "console.log('hello');\nvi.hoisted(() => {});";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`vi.hoisted()` after other code should be flagged"
        );
    }

    #[test]
    fn test_allows_hoisted_at_top() {
        let source = "vi.hoisted(() => {});\ntest('works', () => {});";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`vi.hoisted()` at top of file should not be flagged"
        );
    }

    #[test]
    fn test_allows_hoisted_after_imports() {
        let source = "import { vi } from 'vitest';\nvi.hoisted(() => {});";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`vi.hoisted()` after imports should not be flagged"
        );
    }
}
