//! Rule: `typescript/no-useless-empty-export`
//!
//! Disallow empty `export {}` when there are already other exports or imports
//! in the file. An empty `export {}` is only useful to turn a script file into
//! a module. When the file already has `import` or `export` statements, the
//! empty export is redundant and should be removed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-useless-empty-export";

/// Flags `export {}` statements when the file already contains other
/// `export` or `import` statements.
#[derive(Debug)]
pub struct NoUselessEmptyExport;

impl NativeRule for NoUselessEmptyExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow empty `export {}` when the file already has exports or imports"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let empty_exports = find_empty_exports(source);

        if empty_exports.is_empty() {
            return;
        }

        // Check if there are other export/import statements besides the empty ones
        if has_other_module_statements(source) {
            for (start, end) in empty_exports {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message:
                        "Empty `export {}` is unnecessary when the file already has exports or imports"
                            .to_owned(),
                    span: Span::new(start, end),
                    severity: Severity::Warning,
                    help: Some("Remove the empty `export {}`".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove empty `export {}`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(start, end),
                            replacement: String::new(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

/// Find all `export {}` (empty export) occurrences in the source.
///
/// Returns a list of `(start_offset, end_offset)` for each match.
fn find_empty_exports(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let pattern = "export";
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(pattern)) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_keyword = absolute_pos.saturating_add(pattern.len());
        let remaining = source.get(after_keyword..).unwrap_or("");

        // Skip whitespace after `export`
        let trimmed = remaining.trim_start();
        let ws_len = remaining.len().saturating_sub(trimmed.len());

        // Check if followed by `{}`  (with optional whitespace inside)
        if trimmed.starts_with('{') {
            let brace_content_start = 1; // skip `{`
            let inner = trimmed.get(brace_content_start..).unwrap_or("");
            let inner_trimmed = inner.trim_start();

            if inner_trimmed.starts_with('}') {
                // This is `export {}`
                let inner_ws = inner.len().saturating_sub(inner_trimmed.len());
                // Calculate end: export + ws + { + inner_ws + }
                let end_offset = after_keyword
                    .saturating_add(ws_len)
                    .saturating_add(brace_content_start)
                    .saturating_add(inner_ws)
                    .saturating_add(1); // closing `}`

                // Also include trailing semicolon if present
                let after_brace = source.get(end_offset..).unwrap_or("");
                let final_end = if after_brace.starts_with(';') {
                    end_offset.saturating_add(1)
                } else {
                    end_offset
                };

                let start_u32 = u32::try_from(absolute_pos).unwrap_or(0);
                let end_u32 = u32::try_from(final_end).unwrap_or(start_u32);
                results.push((start_u32, end_u32));
            }
        }

        search_from = after_keyword;
    }

    results
}

/// Check if the source contains `export` or `import` statements other than
/// empty `export {}`.
///
/// Returns `true` if any non-empty export or any import statement is found.
fn has_other_module_statements(source: &str) -> bool {
    // Check for import statements
    if has_keyword_occurrence(source, "import") {
        return true;
    }

    // Check for non-empty export statements
    let pattern = "export";
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(pattern)) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_keyword = absolute_pos.saturating_add(pattern.len());
        let remaining = source.get(after_keyword..).unwrap_or("");
        let trimmed = remaining.trim_start();

        // If followed by `{}` — this is an empty export, skip it
        if is_empty_braces(trimmed) {
            search_from = after_keyword;
            continue;
        }

        // Any other export statement found
        if !trimmed.is_empty() {
            return true;
        }

        search_from = after_keyword;
    }

    false
}

/// Check if a string starts with an `import` keyword that looks like a
/// module statement (not inside another word).
fn has_keyword_occurrence(source: &str, keyword: &str) -> bool {
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(keyword)) {
        let absolute_pos = search_from.saturating_add(pos);

        // Ensure this is not part of a larger identifier
        let before_ok = absolute_pos == 0
            || source
                .as_bytes()
                .get(absolute_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');

        let after_pos = absolute_pos.saturating_add(keyword.len());
        let after_ok = source
            .as_bytes()
            .get(after_pos)
            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');

        if before_ok && after_ok {
            return true;
        }

        search_from = after_pos;
    }

    false
}

/// Check if a string starts with `{}` (with optional whitespace inside).
fn is_empty_braces(s: &str) -> bool {
    if !s.starts_with('{') {
        return false;
    }
    let inner = s.get(1..).unwrap_or("");
    let trimmed = inner.trim_start();
    trimmed.starts_with('}')
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessEmptyExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_export_with_other_export() {
        let diags = lint("export const x = 1;\nexport {};");
        assert_eq!(
            diags.len(),
            1,
            "empty `export` with another export should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_export_with_import() {
        let diags = lint("import { x } from 'y';\nexport {};");
        assert_eq!(
            diags.len(),
            1,
            "empty `export` with an import should be flagged"
        );
    }

    #[test]
    fn test_allows_empty_export_alone() {
        let diags = lint("export {};");
        assert!(
            diags.is_empty(),
            "empty `export` alone (needed for module) should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_empty_export() {
        let diags = lint("export const x = 1;");
        assert!(
            diags.is_empty(),
            "file without empty `export` should not be flagged"
        );
    }

    #[test]
    fn test_allows_export_with_specifiers() {
        let diags = lint("const x = 1;\nexport { x };");
        assert!(
            diags.is_empty(),
            "non-empty `export` with specifiers should not be flagged"
        );
    }
}
