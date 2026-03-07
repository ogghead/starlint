//! Rule: `vue/no-expose-after-await`
//!
//! Forbid calling `expose()` after `await` in `setup()`. When `expose()` is
//! called after an `await`, the component instance context may have changed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-expose-after-await";

/// Forbid `expose()` after `await` in `setup()`.
#[derive(Debug)]
pub struct NoExposeAfterAwait;

impl LintRule for NoExposeAfterAwait {
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

    fn run_once(&self, ctx: &mut LintContext<'_>) {
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
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExposeAfterAwait)];
        lint_source(source, "test.js", &rules)
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
