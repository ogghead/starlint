//! Rule: `import/no-restricted-imports`
//!
//! Forbid specific modules when loaded by import.
//! This is a stub rule — the list of restricted modules would be provided
//! via configuration.

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::LintRule;

/// Stub: would forbid specific configured modules from being imported.
#[derive(Debug)]
pub struct NoRestrictedImports;

impl LintRule for NoRestrictedImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-restricted-imports".to_owned(),
            description: "Forbid specific modules when loaded by import".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRestrictedImports)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_stub_does_not_flag_any_import() {
        let diags = lint(r#"import foo from "lodash";"#);
        assert!(
            diags.is_empty(),
            "stub rule should not produce diagnostics without configuration"
        );
    }

    #[test]
    fn test_stub_does_not_flag_named_import() {
        let diags = lint(r#"import { get } from "lodash";"#);
        assert!(
            diags.is_empty(),
            "stub rule should not produce diagnostics without configuration"
        );
    }
}
