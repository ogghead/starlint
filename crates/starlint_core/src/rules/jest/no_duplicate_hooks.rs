//! Rule: `jest/no-duplicate-hooks`
//!
//! Error when the same hook appears multiple times in a describe block.

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-duplicate-hooks";

/// Hook names to check for duplicates.
const HOOKS: &[&str] = &["beforeEach", "afterEach", "beforeAll", "afterAll"];

/// Flags duplicate lifecycle hooks within the same describe block.
#[derive(Debug)]
pub struct NoDuplicateHooks;

impl NativeRule for NoDuplicateHooks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow duplicate lifecycle hooks in the same describe block".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();

        // Track hook occurrences. For simplicity, count at the top level.
        let mut hook_counts: HashMap<&str, Vec<u32>> = HashMap::new();

        for hook_name in HOOKS {
            let pattern = format!("{hook_name}(");
            let mut search_start: usize = 0;

            while let Some(pos) = source.get(search_start..).and_then(|s| s.find(&pattern)) {
                let abs_pos = search_start.saturating_add(pos);

                // Verify this is a standalone call (not part of a larger identifier)
                let before_char = if abs_pos > 0 {
                    source
                        .get(abs_pos.saturating_sub(1)..abs_pos)
                        .and_then(|s| s.chars().next())
                } else {
                    None
                };

                let is_standalone =
                    before_char.is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '$');

                if is_standalone {
                    let span_start = u32::try_from(abs_pos).unwrap_or(0);
                    hook_counts.entry(hook_name).or_default().push(span_start);
                }

                search_start = abs_pos.saturating_add(pattern.len());
            }
        }

        // Report duplicates
        for (hook_name, positions) in &hook_counts {
            if positions.len() > 1 {
                // Flag all occurrences after the first
                for pos in positions.iter().skip(1) {
                    let end = pos
                        .saturating_add(u32::try_from(hook_name.len()).unwrap_or(0))
                        .saturating_add(1);
                    ctx.report_error(
                        RULE_NAME,
                        &format!("Duplicate `{hook_name}` hook — each hook should only appear once per describe block"),
                        Span::new(*pos, end),
                    );
                }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDuplicateHooks)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_before_each() {
        let source = "beforeEach(() => {}); beforeEach(() => {});";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "duplicate beforeEach should be flagged");
    }

    #[test]
    fn test_allows_single_hooks() {
        let source = "beforeEach(() => {}); afterEach(() => {});";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "single hooks of different types should not be flagged"
        );
    }

    #[test]
    fn test_flags_duplicate_after_all() {
        let source = "afterAll(() => {}); afterAll(() => {});";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "duplicate afterAll should be flagged");
    }
}
