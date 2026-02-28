//! Rule: `vue/no-setup-props-reactivity-loss`
//!
//! Warn about losing reactivity by destructuring `props` in `setup()`.
//! Destructuring props creates plain local variables that will not update
//! when the parent changes the prop values.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-setup-props-reactivity-loss";

/// Warn about losing reactivity by destructuring `props` in `setup()`.
#[derive(Debug)]
pub struct NoSetupPropsReactivityLoss;

impl NativeRule for NoSetupPropsReactivityLoss {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn about losing reactivity by destructuring `props` in `setup()`"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
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
                ctx.report_warning(
                    RULE_NAME,
                    "Destructuring `props` in `setup()` loses reactivity — use `toRefs(props)` or access `props.x` directly",
                    Span::new(start, end),
                );
            }

            search_pos = abs_in_setup.saturating_add(9);
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSetupPropsReactivityLoss)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
