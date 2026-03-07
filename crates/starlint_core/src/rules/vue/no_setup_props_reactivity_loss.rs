//! Rule: `vue/no-setup-props-reactivity-loss`
//!
//! Warn about losing reactivity by destructuring `props` in `setup()`.
//! Destructuring props creates plain local variables that will not update
//! when the parent changes the prop values.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-setup-props-reactivity-loss";

/// Warn about losing reactivity by destructuring `props` in `setup()`.
#[derive(Debug)]
pub struct NoSetupPropsReactivityLoss;

impl LintRule for NoSetupPropsReactivityLoss {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn about losing reactivity by destructuring `props` in `setup()`"
                .to_owned(),
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

        // Look for destructuring patterns from props: `const { x } = props`
        // or `{ x } = props` within setup
        let mut search_pos = 0;
        while let Some(offset) = setup_body
            .get(search_pos..)
            .and_then(|s| s.find("} = props"))
        {
            let abs_in_setup = search_pos.saturating_add(offset);
            let abs_pos = setup_pos.saturating_add(abs_in_setup);

            // Verify there's a `{` before the `}` on the same statement
            let before = setup_body.get(..abs_in_setup).unwrap_or_default();
            if before.contains('{') {
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = start.saturating_add(9); // "} = props" length
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Destructuring `props` in `setup()` loses reactivity — use `toRefs(props)` or access `props.x` directly".to_owned(),
                    span: Span::new(start, end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }

            search_pos = abs_in_setup.saturating_add(9);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoSetupPropsReactivityLoss)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_destructured_props() {
        let source =
            r"export default { setup(props) { const { title } = props; return { title }; } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "destructuring props should be flagged");
    }

    #[test]
    fn test_allows_direct_prop_access() {
        let source =
            r"export default { setup(props) { const title = props.title; return { title }; } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "direct prop access should be allowed");
    }

    #[test]
    fn test_allows_to_refs() {
        let source = r"export default { setup(props) { const { title } = toRefs(props); } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "toRefs destructuring should be allowed");
    }
}
