//! Rule: `jest/no-identical-title`
//!
//! Error when sibling `describe`/`it`/`test` blocks share the same title.
//! Simplified: scans source for duplicate string titles in test calls at the
//! top level.

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-identical-title";

/// Test block identifiers to check for duplicate titles.
const TEST_BLOCKS: &[&str] = &["describe", "it", "test"];

/// Flags sibling test/describe blocks that share identical titles.
#[derive(Debug)]
pub struct NoIdenticalTitle;

impl NativeRule for NoIdenticalTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow identical titles in sibling test blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let violations = find_duplicate_titles(ctx.source_text());

        for (message, span) in violations {
            ctx.report_error(RULE_NAME, &message, span);
        }
    }
}

/// Find duplicate test block titles and return violation messages with spans.
fn find_duplicate_titles(source: &str) -> Vec<(String, Span)> {
    let mut results = Vec::new();

    for block_name in TEST_BLOCKS {
        let mut seen_titles: HashSet<String> = HashSet::new();
        let pattern = format!("{block_name}(");

        let mut search_start: usize = 0;
        while let Some(pos) = source.get(search_start..).and_then(|s| s.find(&pattern)) {
            let abs_pos = search_start.saturating_add(pos);
            let after_paren = abs_pos.saturating_add(pattern.len());

            if let Some(title_info) = extract_string_title(source, after_paren) {
                if !seen_titles.insert(title_info.title.clone()) {
                    results.push((
                        format!("Duplicate {block_name} title: \"{}\"", title_info.title),
                        Span::new(
                            u32::try_from(title_info.span_start).unwrap_or(0),
                            u32::try_from(title_info.span_end).unwrap_or(0),
                        ),
                    ));
                }
            }

            search_start = after_paren.saturating_add(1);
        }
    }

    results
}

/// Information about an extracted title string.
struct TitleInfo {
    /// The title text.
    title: String,
    /// Byte offset of the title start in the source.
    span_start: usize,
    /// Byte offset of the title end in the source.
    span_end: usize,
}

/// Extract a string literal title from the source at the given position.
fn extract_string_title(source: &str, pos: usize) -> Option<TitleInfo> {
    let remaining = source.get(pos..)?;
    let trimmed = remaining.trim_start();
    let skip = remaining.len().saturating_sub(trimmed.len());
    let quote_pos = pos.saturating_add(skip);

    let quote = trimmed.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }

    let after_quote = trimmed.get(1..)?;
    let end_idx = after_quote.find(quote)?;
    let title = after_quote.get(..end_idx)?;

    Some(TitleInfo {
        title: title.to_owned(),
        span_start: quote_pos,
        // +2 for the two quote characters
        span_end: quote_pos.saturating_add(end_idx).saturating_add(2),
    })
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoIdenticalTitle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_test_titles() {
        let source = r"test('do thing', () => {}); test('do thing', () => {});";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "duplicate test titles should be flagged");
    }

    #[test]
    fn test_allows_unique_titles() {
        let source = r"test('first', () => {}); test('second', () => {});";
        let diags = lint(source);
        assert!(diags.is_empty(), "unique titles should not be flagged");
    }

    #[test]
    fn test_flags_duplicate_describe_titles() {
        let source = r"describe('suite', () => {}); describe('suite', () => {});";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "duplicate describe titles should be flagged"
        );
    }
}
