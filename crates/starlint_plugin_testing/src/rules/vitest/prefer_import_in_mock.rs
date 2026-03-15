//! Rule: `vitest/prefer-import-in-mock`
//!
//! Suggest using `vi.importActual` inside `vi.mock` factory functions instead
//! of `require`. When partially mocking a module, the factory function passed
//! to `vi.mock` should use `await vi.importActual(...)` to get the real
//! module rather than `require(...)`, because `require` bypasses Vitest's
//! module resolution and can lead to inconsistencies.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-import-in-mock";

/// Suggest `vi.importActual` over `require` inside `vi.mock` factories.
#[derive(Debug)]
pub struct PreferImportInMock;

impl LintRule for PreferImportInMock {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Prefer `vi.importActual` over `require` inside `vi.mock` factory functions"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("vi.mock") && crate::is_test_file(file_path)
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations = find_require_in_mock_factory(ctx.source_text());

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Use `await vi.importActual(...)` instead of `require(...)` inside `vi.mock` factory".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace with `await vi.importActual(`".to_owned(),
                    edits: vec![Edit {
                        span,
                        replacement: "await vi.importActual(".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Find `require(` calls inside `vi.mock` factory bodies.
fn find_require_in_mock_factory(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let mock_needle = "vi.mock(";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(mock_needle)) {
        let abs_pos = search_start.saturating_add(pos);
        let after_mock = abs_pos.saturating_add(mock_needle.len());

        if let Some(rest) = source.get(after_mock..) {
            if let Some(brace_offset) = rest.find('{') {
                let body_start = after_mock.saturating_add(brace_offset);

                if let Some(body_end) = find_matching_brace(source, body_start) {
                    let body = source.get(body_start..body_end).unwrap_or("");

                    if let Some(req_pos) = body.find("require(") {
                        let abs_req = body_start.saturating_add(req_pos);
                        let start = u32::try_from(abs_req).unwrap_or(0);
                        let end = u32::try_from(abs_req.saturating_add("require(".len()))
                            .unwrap_or(start);
                        results.push(Span::new(start, end));
                    }
                }
            }
        }

        search_start = abs_pos.saturating_add(1);
    }

    results
}

/// Find the matching closing brace for the brace at `open_pos`.
fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    let mut depth: usize = 0;
    for (i, ch) in source.get(open_pos..)?.char_indices() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_pos.saturating_add(i).saturating_add(1));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(PreferImportInMock);

    #[test]
    fn test_flags_require_in_mock_factory() {
        let source = r#"vi.mock("./module", () => { const actual = require("./module"); return { ...actual }; });"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`require` inside `vi.mock` factory should be flagged"
        );
    }

    #[test]
    fn test_allows_import_actual_in_mock() {
        let source = r#"vi.mock("./module", async () => { const actual = await vi.importActual("./module"); return { ...actual }; });"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`vi.importActual` inside `vi.mock` should not be flagged"
        );
    }

    #[test]
    fn test_allows_vi_mock_without_factory() {
        let source = r#"vi.mock("./module");"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`vi.mock` without factory should not be flagged"
        );
    }
}
