//! Rule: `vue/no-lifecycle-after-await`
//!
//! Forbid lifecycle hooks (`onMounted`, `onUpdated`, `onUnmounted`, etc.)
//! after `await` in `setup()`. Lifecycle hooks must be registered synchronously.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-lifecycle-after-await";

/// Lifecycle hooks from the Vue 3 Composition API.
const LIFECYCLE_HOOKS: &[&str] = &[
    "onMounted",
    "onUpdated",
    "onUnmounted",
    "onBeforeMount",
    "onBeforeUpdate",
    "onBeforeUnmount",
    "onErrorCaptured",
    "onRenderTracked",
    "onRenderTriggered",
    "onActivated",
    "onDeactivated",
    "onServerPrefetch",
];

/// Forbid lifecycle hooks after `await` in `setup()`.
#[derive(Debug)]
pub struct NoLifecycleAfterAwait;

impl NativeRule for NoLifecycleAfterAwait {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid lifecycle hooks after `await` in `setup()`".to_owned(),
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

        let after_await = setup_body.get(await_offset..).unwrap_or_default();

        for hook in LIFECYCLE_HOOKS {
            let pattern = format!("{hook}(");
            if let Some(hook_offset) = after_await.find(pattern.as_str()) {
                let abs_pos = setup_pos
                    .saturating_add(await_offset)
                    .saturating_add(hook_offset);
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = start.saturating_add(u32::try_from(hook.len()).unwrap_or(0));
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`{hook}` should not be called after `await` in `setup()` — lifecycle hooks must be registered synchronously"
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLifecycleAfterAwait)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_on_mounted_after_await() {
        let source = r"export default { setup() { await fetchData(); onMounted(() => {}); } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "onMounted after await should be flagged");
    }

    #[test]
    fn test_allows_on_mounted_before_await() {
        let source = r"export default { setup() { onMounted(() => {}); await fetchData(); } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "onMounted before await should be allowed");
    }

    #[test]
    fn test_flags_on_updated_after_await() {
        let source = r"export default { setup() { await fetchData(); onUpdated(() => {}); } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "onUpdated after await should be flagged");
    }
}
