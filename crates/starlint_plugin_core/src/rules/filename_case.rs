//! Rule: `filename-case`
//!
//! Enforce consistent filename casing convention. By default, enforce
//! kebab-case: all lowercase, words separated by hyphens, no underscores
//! or uppercase characters. Skips common entry-point names like `index`,
//! `lib`, `main`, and dotfiles.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::case_utils::is_kebab_case;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags filenames that do not follow kebab-case convention.
#[derive(Debug)]
pub struct FilenameCase;

/// Filenames that are exempt from the kebab-case check.
const EXEMPT_STEMS: &[&str] = &["index", "lib", "main"];

impl LintRule for FilenameCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "filename-case".to_owned(),
            description: "Enforce consistent filename casing convention".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let file_path = ctx.file_path();
        let Some(stem_os) = file_path.file_stem() else {
            return;
        };
        let stem = stem_os.to_string_lossy();

        // Skip dotfiles (stems starting with `.`)
        if stem.starts_with('.') {
            return;
        }

        // Skip common entry-point names
        if EXEMPT_STEMS.contains(&stem.as_ref()) {
            return;
        }

        if !is_kebab_case(&stem) {
            ctx.report(Diagnostic {
                rule_name: "filename-case".to_owned(),
                message:
                    "Filename should be in kebab-case (lowercase with hyphens, no underscores)"
                        .to_owned(),
                span: Span::new(0, 0),
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
    use starlint_rule_framework::lint_source;
    starlint_rule_framework::lint_rule_test!(FilenameCase);

    fn lint_with_path(
        source: &str,
        path: &str,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(FilenameCase)];
        lint_source(source, path, &rules)
    }

    #[test]
    fn test_allows_kebab_case() {
        let diags = lint_with_path("var x = 1;", "foo-bar.js");
        assert!(
            diags.is_empty(),
            "kebab-case filename should not be flagged"
        );
    }

    #[test]
    fn test_flags_pascal_case() {
        let diags = lint_with_path("var x = 1;", "FooBar.js");
        assert_eq!(diags.len(), 1, "PascalCase filename should be flagged");
    }

    #[test]
    fn test_flags_snake_case() {
        let diags = lint_with_path("var x = 1;", "foo_bar.js");
        assert_eq!(diags.len(), 1, "snake_case filename should be flagged");
    }

    #[test]
    fn test_allows_index() {
        let diags = lint_with_path("var x = 1;", "index.js");
        assert!(diags.is_empty(), "index.js should not be flagged");
    }

    #[test]
    fn test_allows_lib() {
        let diags = lint_with_path("var x = 1;", "lib.js");
        assert!(diags.is_empty(), "lib.js should not be flagged");
    }

    #[test]
    fn test_allows_main() {
        let diags = lint_with_path("var x = 1;", "main.js");
        assert!(diags.is_empty(), "main.js should not be flagged");
    }

    #[test]
    fn test_allows_lowercase() {
        let diags = lint("var x = 1;");
        // "test.js" is kebab-case (single word)
        assert!(diags.is_empty(), "lowercase filename should not be flagged");
    }

    #[test]
    fn test_flags_camel_case() {
        let diags = lint_with_path("var x = 1;", "myComponent.js");
        assert_eq!(diags.len(), 1, "camelCase filename should be flagged");
    }
}
