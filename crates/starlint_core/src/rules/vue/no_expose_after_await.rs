//! Rule: `vue/no-expose-after-await`
//!
//! Forbid calling `expose()` after `await` in `setup()`. When `expose()` is
//! called after an `await`, the component instance context may have changed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-expose-after-await";

/// Forbid `expose()` after `await` in `setup()`.
#[derive(Debug)]
pub struct NoExposeAfterAwait;

impl NativeRule for NoExposeAfterAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid calling `expose()` after `await` in `setup()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Find setup function
        let Some(setup_pos) = source.find("setup") else {
            return;
        };

        let setup_body = source.get(setup_pos..).unwrap_or_default();

        // Find the first `await` in setup
        let Some(await_offset) = setup_body.find("await ") else {
            return;
        };

        // Look for `expose(` after the await
        let after_await = setup_body.get(await_offset..).unwrap_or_default();

        if let Some(expose_offset) = after_await.find("expose(") {
            let abs_pos = setup_pos
                .saturating_add(await_offset)
                .saturating_add(expose_offset);
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(7); // "expose(" length
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`expose()` should not be called after `await` in `setup()` — the component context may have changed".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExposeAfterAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_expose_after_await() {
        let source = r"export default { setup() { await fetchData(); expose({ foo }); } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "expose after await should be flagged");
    }

    #[test]
    fn test_allows_expose_before_await() {
        let source = r"export default { setup() { expose({ foo }); await fetchData(); } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "expose before await should be allowed");
    }

    #[test]
    fn test_no_setup() {
        let source = r"export default { data() { return {}; } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "no setup should produce no diags");
    }
}
