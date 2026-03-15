//! Rule: `storybook/prefer-pascal-case`
//!
//! Stories should use `PascalCase` names.
//! Checks named export identifiers in `.stories.` files for `PascalCase`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/prefer-pascal-case";

/// Stories should use `PascalCase` names.
#[derive(Debug)]
pub struct PreferPascalCase;

/// Convert a string to `PascalCase`.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Check if a string is `PascalCase` (starts with uppercase, no underscores/hyphens at start).
fn is_pascal_case(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };

    // Must start with uppercase letter
    if !first.is_ascii_uppercase() {
        return false;
    }

    // Should not contain underscores or hyphens (allowing `_` for special exports like `__namedExportsOrder`)
    !s.contains('-')
}

impl LintRule for PreferPascalCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Stories should use PascalCase names".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let file_name = ctx.file_path().to_string_lossy();
        if !file_name.contains(".stories.") && !file_name.contains(".story.") {
            return;
        }

        let source = ctx.source_text().to_owned();

        // Find named exports: `export const Name =` or `export let Name =`
        let patterns = ["export const ", "export let "];

        for pattern in &patterns {
            let mut search_pos = 0;
            while let Some(pos) = source.get(search_pos..).and_then(|s| s.find(pattern)) {
                let abs_pos = search_pos.saturating_add(pos);
                let after = source
                    .get(abs_pos.saturating_add(pattern.len())..)
                    .unwrap_or_default();

                // Extract the identifier name
                let name_end = after
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(after.len());
                let export_name = after.get(..name_end).unwrap_or_default();

                // Skip special identifiers like `default` or internal names starting with `__`
                if export_name.is_empty()
                    || export_name == "default"
                    || export_name.starts_with("__")
                {
                    search_pos = abs_pos.saturating_add(1);
                    continue;
                }

                if !is_pascal_case(export_name) {
                    let name_start = abs_pos.saturating_add(pattern.len());
                    let start = u32::try_from(name_start).unwrap_or(0);
                    let end = start.saturating_add(u32::try_from(export_name.len()).unwrap_or(0));
                    let pascal = to_pascal_case(export_name);
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Story exports should use PascalCase".to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: Some(format!("Rename to `{pascal}`")),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Rename to `{pascal}`"),
                            edits: vec![Edit {
                                span: Span::new(start, end),
                                replacement: pascal,
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }

                search_pos = abs_pos.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(PreferPascalCase, "Button.stories.tsx");

    #[test]
    fn test_flags_non_pascal_case() {
        let diags = lint("export default {}; export const primary = {};");
        assert_eq!(diags.len(), 1, "should flag non-PascalCase export");
    }

    #[test]
    fn test_allows_pascal_case() {
        let diags = lint("export default {}; export const Primary = {};");
        assert!(diags.is_empty(), "should allow PascalCase export");
    }

    #[test]
    fn test_allows_multi_word_pascal_case() {
        let diags = lint("export default {}; export const PrimaryButton = {};");
        assert!(diags.is_empty(), "should allow multi-word PascalCase");
    }
}
