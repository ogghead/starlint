//! Rule: `storybook/no-uninstalled-addons`
//!
//! Identifies storybook addons that aren't installed.
//! Text-based stub: checks for `addons:` array referencing common addon packages.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/no-uninstalled-addons";

/// Common storybook addon package prefixes.
const ADDON_PREFIXES: &[&str] = &["@storybook/addon-", "storybook-addon-"];

/// Identifies storybook addons that aren't installed.
#[derive(Debug)]
pub struct NoUninstalledAddons;

impl NativeRule for NoUninstalledAddons {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Identifies storybook addons that are not installed".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let file_name = ctx.file_path().to_string_lossy();

        // This rule applies to storybook config files (main.js/ts) or story files
        let is_storybook_config = file_name.contains(".storybook") && file_name.contains("main");
        if !is_storybook_config {
            return;
        }

        let source = ctx.source_text().to_owned();

        // Look for addons array references
        if !source.contains("addons") {
            return;
        }

        // Find addon string references and flag them as a warning stub
        // In a full implementation, this would cross-reference with package.json
        for prefix in ADDON_PREFIXES {
            let mut search_pos = 0;
            while let Some(pos) = source.get(search_pos..).and_then(|s| s.find(prefix)) {
                let abs_pos = search_pos.saturating_add(pos);
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = start.saturating_add(u32::try_from(prefix.len()).unwrap_or(0));

                // Find the full addon name (up to next quote)
                let remaining = source.get(abs_pos..).unwrap_or_default();
                let addon_end = remaining.find(['\'', '"', '`']).unwrap_or(prefix.len());
                let addon_name = remaining.get(..addon_end).unwrap_or_default();
                let end_full = start.saturating_add(u32::try_from(addon_name.len()).unwrap_or(0));

                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Verify that this storybook addon is installed in your dependencies"
                        .to_owned(),
                    span: Span::new(start, end_full),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });

                let _ = end;
                search_pos = abs_pos.saturating_add(1);
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUninstalledAddons)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_addon_in_config() {
        let diags = lint_with_path(
            "module.exports = { addons: ['@storybook/addon-essentials'] };",
            Path::new(".storybook/main.ts"),
        );
        assert_eq!(diags.len(), 1, "should flag addon reference in config");
    }

    #[test]
    fn test_ignores_non_config_files() {
        let diags = lint_with_path(
            "const addons = ['@storybook/addon-essentials'];",
            Path::new("Button.stories.tsx"),
        );
        assert!(diags.is_empty(), "should ignore non-config files");
    }

    #[test]
    fn test_allows_no_addons() {
        let diags = lint_with_path(
            "module.exports = { stories: ['../src/**/*.stories.@(js|jsx|ts|tsx)'] };",
            Path::new(".storybook/main.ts"),
        );
        assert!(diags.is_empty(), "should allow config without addons");
    }
}
