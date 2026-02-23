//! Rule: `vue/no-watch-after-await`
//!
//! Forbid `watch()` / `watchEffect()` after `await` in `setup()`.
//! Watchers registered after an `await` may not be properly cleaned up
//! when the component is unmounted.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-watch-after-await";

/// Watch functions from the Vue 3 Composition API.
const WATCH_FUNCTIONS: &[&str] = &[
    "watch(",
    "watchEffect(",
    "watchPostEffect(",
    "watchSyncEffect(",
];

/// Forbid `watch()` after `await` in `setup()`.
#[derive(Debug)]
pub struct NoWatchAfterAwait;

impl LintRule for NoWatchAfterAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `watch()` after `await` in `setup()`".to_owned(),
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

        let after_await = setup_body.get(await_offset..).unwrap_or_default();

        for pattern in WATCH_FUNCTIONS {
            if let Some(watch_offset) = after_await.find(pattern) {
                let func_name = pattern.trim_end_matches('(');
                let abs_pos = setup_pos
                    .saturating_add(await_offset)
                    .saturating_add(watch_offset);
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = start.saturating_add(u32::try_from(func_name.len()).unwrap_or(0));
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`{func_name}` should not be called after `await` in `setup()` — watchers may not be cleaned up on unmount"
                    ),
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
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoWatchAfterAwait)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_watch_after_await() {
        let source = r"export default { setup() { await fetchData(); watch(source, cb); } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "watch after await should be flagged");
    }

    #[test]
    fn test_allows_watch_before_await() {
        let source = r"export default { setup() { watch(source, cb); await fetchData(); } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "watch before await should be allowed");
    }

    #[test]
    fn test_flags_watch_effect_after_await() {
        let source = r"export default { setup() { await fetchData(); watchEffect(() => {}); } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "watchEffect after await should be flagged");
    }
}
