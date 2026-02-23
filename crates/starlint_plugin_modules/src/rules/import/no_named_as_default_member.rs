//! Rule: `import/no-named-as-default-member`
//!
//! Forbid use of an exported name as a property of the default export.
//! For example, if a module has `export const foo = 1; export default bar`,
//! then `import bar from './mod'; bar.foo` should use `import { foo }` instead.
//!
//! Without full module resolution, this rule is a stub that documents intent.

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::LintRule;

/// Stub: would flag property access on default imports that matches a named export.
#[derive(Debug)]
pub struct NoNamedAsDefaultMember;

impl LintRule for NoNamedAsDefaultMember {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-as-default-member".to_owned(),
            description: "Forbid use of an exported name as a property of the default export"
                .to_owned(),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNamedAsDefaultMember)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_stub_does_not_flag_default_import() {
        let diags = lint(r#"import foo from "./module";"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }

    #[test]
    fn test_stub_does_not_flag_member_access() {
        let diags = lint(r#"import foo from "./module"; console.log(foo.bar);"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }
}
