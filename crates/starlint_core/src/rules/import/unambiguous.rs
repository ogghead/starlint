//! Rule: `import/unambiguous`
//!
//! Warn when a module could be parsed as either a script or a module.
//! In ECMAScript, a file is a module if it contains at least one `import`
//! or `export` statement. Files without these are ambiguous.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags files that contain no `import` or `export` statements.
#[derive(Debug)]
pub struct Unambiguous;

impl LintRule for Unambiguous {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/unambiguous".to_owned(),
            description: "Warn when a module could be parsed as either a script or a module"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let has_module_syntax = {
            let source = ctx.source_text();
            source.lines().any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("import ")
                    || trimmed.starts_with("import{")
                    || trimmed == "import("
                    || trimmed.starts_with("export ")
                    || trimmed.starts_with("export{")
                    || trimmed.starts_with("export default ")
            })
        };

        if !has_module_syntax {
            ctx.report(Diagnostic {
                rule_name: "import/unambiguous".to_owned(),
                message: "This file could be parsed as a script — add an import or export to make it unambiguously a module".to_owned(),
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
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(Unambiguous)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_script_file() {
        let diags = lint("const x = 1;\nconsole.log(x);");
        assert_eq!(
            diags.len(),
            1,
            "file without import/export should be flagged"
        );
    }

    #[test]
    fn test_allows_file_with_import() {
        let diags = lint(r#"import { foo } from "bar"; foo();"#);
        assert!(diags.is_empty(), "file with import should not be flagged");
    }

    #[test]
    fn test_allows_file_with_export() {
        let diags = lint("export const x = 1;");
        assert!(diags.is_empty(), "file with export should not be flagged");
    }
}
