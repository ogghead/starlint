//! Rule: `vue/no-ref-object-reactivity-loss`
//!
//! Warn about losing reactivity by destructuring `ref` objects. When you
//! destructure a `ref()` return value, the resulting variables are plain values,
//! not reactive references.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-ref-object-reactivity-loss";

/// Warn about losing reactivity by destructuring `ref` objects.
#[derive(Debug)]
pub struct NoRefObjectReactivityLoss;

impl LintRule for NoRefObjectReactivityLoss {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn about losing reactivity by destructuring `ref` objects".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Look for patterns like: const { x } = ref(...)  or  const { x } = reactive(...)
        // Also: const { value } = someRef
        // Pattern: destructuring assignment from ref/reactive calls
        for func_name in &["ref(", "reactive(", "toRef(", "toRefs("] {
            let mut search_pos = 0;
            while let Some(offset) = source.get(search_pos..).and_then(|s| s.find(func_name)) {
                let abs_pos = search_pos.saturating_add(offset);

                // Look backwards for destructuring pattern `{ ... } =`
                let before = source.get(..abs_pos).unwrap_or_default();
                let trimmed = before.trim_end();

                // Check if preceded by `= ` (assignment from ref call)
                if trimmed.ends_with('=') {
                    let before_eq = trimmed
                        .get(..trimmed.len().saturating_sub(1))
                        .unwrap_or_default()
                        .trim_end();
                    if before_eq.ends_with('}') {
                        // Found destructuring from ref/reactive
                        let start = u32::try_from(abs_pos).unwrap_or(0);
                        let end = start.saturating_add(u32::try_from(func_name.len()).unwrap_or(0));
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: format!(
                                "Destructuring a `{name}` return value loses reactivity — use `.value` or `toRefs()` instead",
                                name = func_name.trim_end_matches('(')
                            ),
                            span: Span::new(start, end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }

                search_pos = abs_pos.saturating_add(func_name.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRefObjectReactivityLoss)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_destructured_reactive() {
        let source = r"const { count } = reactive({ count: 0 });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "destructuring reactive should be flagged");
    }

    #[test]
    fn test_allows_direct_assignment() {
        let source = r"const count = ref(0);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "direct assignment from ref should be allowed"
        );
    }

    #[test]
    fn test_flags_destructured_ref() {
        let source = r"const { value } = ref(0);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "destructuring ref should be flagged");
    }
}
