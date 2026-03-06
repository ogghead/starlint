//! Rule: `vue/no-async-in-computed-properties`
//!
//! Forbid async functions in computed properties. Computed properties must be
//! synchronous — using `async` causes them to return a `Promise` instead of
//! the expected value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-async-in-computed-properties";

/// Forbid async in computed properties.
#[derive(Debug)]
pub struct NoAsyncInComputedProperties;

impl NativeRule for NoAsyncInComputedProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid async functions in computed properties".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Find computed: { ... } blocks
        let Some(computed_pos) = source.find("computed:") else {
            return;
        };

        let search_start = computed_pos.saturating_add(9);
        let remaining = source.get(search_start..).unwrap_or_default();

        // Look for `async` keyword within the computed block
        // Find the matching closing brace to limit search scope
        let mut brace_depth = 0_i32;
        let mut block_end = remaining.len();
        for (i, ch) in remaining.char_indices() {
            match ch {
                '{' => brace_depth = brace_depth.saturating_add(1),
                '}' => {
                    brace_depth = brace_depth.saturating_sub(1);
                    if brace_depth == 0 {
                        block_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let block = remaining.get(..block_end).unwrap_or_default();
        if let Some(async_offset) = block.find("async") {
            let abs_pos = search_start.saturating_add(async_offset);
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(5);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Computed properties must be synchronous — do not use `async`".to_owned(),
                span: Span::new(start, end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAsyncInComputedProperties)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_async_computed() {
        let source =
            r"export default { computed: { myProp: async function() { return await fetch(); } } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "async computed should be flagged");
    }

    #[test]
    fn test_allows_sync_computed() {
        let source = r"export default { computed: { myProp() { return this.value; } } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "sync computed should be allowed");
    }

    #[test]
    fn test_no_computed_block() {
        let source = r"export default { data() { return {}; } };";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "no computed block should produce no diags"
        );
    }
}
