//! Rule: `nextjs/no-duplicate-head`
//!
//! Forbid duplicate `<Head>` components from `next/head` in a single file.
//! Multiple `<Head>` components can cause unexpected behavior.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-duplicate-head";

/// Flags duplicate `<Head>` component usage by scanning source text for
/// multiple occurrences.
#[derive(Debug)]
pub struct NoDuplicateHead;

impl NativeRule for NoDuplicateHead {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid duplicate `<Head>` components".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();

        // Collect positions of all `<Head` occurrences (JSX opening tags)
        let positions: Vec<usize> = source
            .match_indices("<Head")
            .filter(|(pos, _)| {
                // Make sure it's followed by a space, > or / (not <Header etc.)
                let after = source.get(pos.saturating_add(5)..pos.saturating_add(6));
                matches!(after, Some(" " | ">" | "/") | None)
            })
            .map(|(pos, _)| pos)
            .collect();

        // Only flag if there are more than one
        if positions.len() > 1 {
            // Flag all occurrences after the first
            for pos in positions.into_iter().skip(1) {
                let start = u32::try_from(pos).unwrap_or(0);
                let end = u32::try_from(pos.saturating_add(5)).unwrap_or(0);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Duplicate `<Head>` component found -- only one `<Head>` should be used per page".to_owned(),
                    span: Span::new(start, end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDuplicateHead)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_head() {
        let source = r"const el = <><Head><title>A</title></Head><Head><title>B</title></Head></>;";

        let diags = lint(source);
        assert_eq!(diags.len(), 1, "duplicate Head should be flagged");
    }

    #[test]
    fn test_allows_single_head() {
        let source = r"const el = <Head><title>Hello</title></Head>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "single Head should not be flagged");
    }

    #[test]
    fn test_ignores_header_element() {
        let source = r"const el = <><Header /><Header /></>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "Header elements should not be flagged");
    }
}
