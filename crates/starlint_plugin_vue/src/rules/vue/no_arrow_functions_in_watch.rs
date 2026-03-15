//! Rule: `vue/no-arrow-functions-in-watch`
//!
//! Forbid arrow functions in the `watch` option.
//! Arrow functions bind `this` lexically, so `this` will not refer to the
//! component instance inside a watcher.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-arrow-functions-in-watch";

/// Forbid arrow functions in the `watch` option.
#[derive(Debug)]
pub struct NoArrowFunctionsInWatch;

impl LintRule for NoArrowFunctionsInWatch {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid arrow functions in the `watch` option".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Find watch: { ... } blocks
        let Some(watch_pos) = source.find("watch:") else {
            return;
        };

        // Look for arrow functions within the watch block
        let search_start = watch_pos.saturating_add(6);
        let remaining = source.get(search_start..).unwrap_or_default();

        if let Some(arrow_offset) = remaining.find("=>") {
            let abs_pos = search_start.saturating_add(arrow_offset);
            let start = u32::try_from(abs_pos).unwrap_or(0);
            let end = start.saturating_add(2);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use arrow functions in `watch` — `this` will not refer to the component instance".to_owned(),
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
    starlint_rule_framework::lint_rule_test!(NoArrowFunctionsInWatch);

    #[test]
    fn test_flags_arrow_in_watch() {
        let source = r"export default { watch: { value: (val) => console.log(val) } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "arrow in watch should be flagged");
    }

    #[test]
    fn test_allows_function_in_watch() {
        let source = r"export default { watch: { value(val) { console.log(val); } } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "function in watch should be allowed");
    }

    #[test]
    fn test_no_watch_block() {
        let source = r"export default { data() { return {}; } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "no watch block should produce no diags");
    }
}
