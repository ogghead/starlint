//! Rule: `import/no-named-as-default`
//!
//! Forbid use of an exported name as the default import identifier.
//! This is a common mistake when a module exports both a default and named
//! exports, and the developer accidentally uses a named export's name as
//! the default import.
//!
//! Without full module resolution, this rule is a stub that documents intent.

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::LintRule;

/// Stub: would flag default imports that share a name with a named export.
#[derive(Debug)]
pub struct NoNamedAsDefault;

impl LintRule for NoNamedAsDefault {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-as-default".to_owned(),
            description: "Forbid use of an exported name as the default import".to_owned(),
            category: Category::Correctness,
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
    starlint_rule_framework::lint_rule_test!(NoNamedAsDefault);

    #[test]
    fn test_stub_does_not_flag_default_import() {
        let diags = lint(r#"import Foo from "./module";"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }

    #[test]
    fn test_stub_does_not_flag_named_import() {
        let diags = lint(r#"import { Foo } from "./module";"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }
}
