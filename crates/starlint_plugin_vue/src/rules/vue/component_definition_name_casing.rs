//! Rule: `vue/component-definition-name-casing`
//!
//! Enforce `PascalCase` or kebab-case for component definition names.
//! Scans for `name:` inside `defineComponent()` or `export default { name: }`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/component-definition-name-casing";

/// Enforce `PascalCase` or kebab-case for component definition names.
#[derive(Debug)]
pub struct ComponentDefinitionNameCasing;

/// Check if a string is `PascalCase` (starts uppercase, no hyphens).
fn is_pascal_case(s: &str) -> bool {
    let first = s.chars().next();
    matches!(first, Some('A'..='Z')) && !s.contains('-')
}

/// Convert a string to `PascalCase`.
fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.collect();
                    format!("{upper}{rest}")
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Check if a string is kebab-case (all lowercase with hyphens).
fn is_kebab_case(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
        && !s.starts_with('-')
        && !s.ends_with('-')
}

impl LintRule for ComponentDefinitionNameCasing {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce PascalCase or kebab-case for component definition names"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Search for `name:` followed by a string literal
        let mut search_start = 0;
        while let Some(offset) = source.get(search_start..).and_then(|s| s.find("name:")) {
            let abs_pos = search_start.saturating_add(offset);
            let after_name = source
                .get(abs_pos.saturating_add(5)..)
                .unwrap_or_default()
                .trim_start();

            // Extract the quoted name value
            let (quote, rest) = if let Some(b'\'' | b'"') = after_name.as_bytes().first() {
                let q = after_name.as_bytes().first().copied().unwrap_or(b'"');
                (char::from(q), after_name.get(1..).unwrap_or_default())
            } else {
                search_start = abs_pos.saturating_add(5);
                continue;
            };

            let end_quote = rest.find(quote).unwrap_or(0);
            let name_value = rest.get(..end_quote).unwrap_or_default();

            if !name_value.is_empty() && !is_pascal_case(name_value) && !is_kebab_case(name_value) {
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = start.saturating_add(
                    u32::try_from(5_usize.saturating_add(2).saturating_add(end_quote)).unwrap_or(0),
                );

                // Fix: convert to PascalCase
                let pascal = to_pascal_case(name_value);
                let fix = (pascal != name_value).then(|| {
                    // Find the name value span (inside the quotes)
                    let name_offset = abs_pos
                        .saturating_add(5)
                        .saturating_add(
                            source
                                .get(abs_pos.saturating_add(5)..)
                                .map_or(0, |s| s.len().saturating_sub(s.trim_start().len())),
                        )
                        .saturating_add(1); // skip opening quote
                    let name_start = u32::try_from(name_offset).unwrap_or(0);
                    let name_end =
                        name_start.saturating_add(u32::try_from(name_value.len()).unwrap_or(0));
                    Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Rename to `{pascal}`"),
                        edits: vec![Edit {
                            span: Span::new(name_start, name_end),
                            replacement: pascal.clone(),
                        }],
                        is_snippet: false,
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Component name `{name_value}` should be PascalCase or kebab-case"
                    ),
                    span: Span::new(start, end),
                    severity: Severity::Warning,
                    help: Some(format!("Rename to `{pascal}`")),
                    fix,
                    labels: vec![],
                });
            }

            search_start = abs_pos.saturating_add(5);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ComponentDefinitionNameCasing)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_pascal_case() {
        let source = r#"export default defineComponent({ name: "MyComponent" });"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "PascalCase name should be allowed");
    }

    #[test]
    fn test_allows_kebab_case() {
        let source = r#"export default defineComponent({ name: "my-component" });"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "kebab-case name should be allowed");
    }

    #[test]
    fn test_flags_bad_casing() {
        let source = r#"export default defineComponent({ name: "myComponent" });"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "camelCase name should be flagged");
    }
}
