//! Rule: `import/no-named-default`
//!
//! Forbid named default exports (`export { default }`).
//! Prefer `export default` syntax for clarity.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Extract the alias name from `import { default as Foo }` pattern.
fn extract_default_alias(line: &str) -> Option<String> {
    let after = line
        .find("default as ")?
        .saturating_add("default as ".len());
    let rest = line.get(after..)?;
    // Alias is the next word (identifier)
    let alias_end = rest.find([' ', '}', ','])?;
    let alias = rest.get(..alias_end)?.trim();
    (!alias.is_empty()).then(|| alias.to_owned())
}

/// Flags re-exports of the `default` name via named export syntax.
#[derive(Debug)]
pub struct NoNamedDefault;

impl LintRule for NoNamedDefault {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-default".to_owned(),
            description: "Forbid named default exports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();
        let findings: Vec<(u32, u32, String)> = source
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let trimmed = line.trim();
                (trimmed.starts_with("import ") && trimmed.contains("{ default ")).then(|| {
                    let line_offset: usize = source
                        .lines()
                        .take(idx)
                        .map(|l| l.len().saturating_add(1))
                        .sum();
                    let start = u32::try_from(line_offset).unwrap_or(0);
                    let end = u32::try_from(line_offset.saturating_add(trimmed.len())).unwrap_or(0);
                    (start, end, trimmed.to_owned())
                })
            })
            .collect();

        for (start, end, line_text) in findings {
            // Try to extract the alias name from `{ default as Foo }`
            let fix = extract_default_alias(&line_text).map(|alias| {
                // Replace `{ default as Foo }` with just `Foo`
                let replacement = line_text.replace(&format!("{{ default as {alias} }}"), &alias);
                Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Use `import {alias} from ...`"),
                    edits: vec![Edit {
                        span: Span::new(start, end),
                        replacement,
                    }],
                    is_snippet: false,
                }
            });

            ctx.report(Diagnostic {
                rule_name: "import/no-named-default".to_owned(),
                message: "Use default import syntax instead of named `default` import".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Use `import Name from ...` instead".to_owned()),
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNamedDefault)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_named_default_import() {
        let diags = lint(r#"import { default as foo } from "mod";"#);
        assert_eq!(diags.len(), 1, "named default import should be flagged");
    }

    #[test]
    fn test_allows_regular_default_import() {
        let diags = lint(r#"import foo from "mod";"#);
        assert!(
            diags.is_empty(),
            "regular default import should not be flagged"
        );
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "mod";"#);
        assert!(diags.is_empty(), "named import should not be flagged");
    }
}
