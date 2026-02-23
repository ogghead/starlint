//! Rule: `import/first`
//!
//! Ensure all imports appear before other statements. Import declarations
//! should be at the top of the file (after comments and directives) to
//! make dependencies immediately visible.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags import declarations that appear after non-import statements.
#[derive(Debug)]
pub struct First;

impl LintRule for First {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/first".to_owned(),
            description: "Ensure all imports appear before other statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let findings: Vec<(u32, u32)> = {
            let source = ctx.source_text();
            let mut found_non_import = false;
            let mut byte_offset: usize = 0;
            let mut results = Vec::new();

            for line in source.lines() {
                let trimmed = line.trim();

                // Skip empty lines and comments
                if trimmed.is_empty()
                    || trimmed.starts_with("//")
                    || trimmed.starts_with("/*")
                    || trimmed.starts_with('*')
                    || trimmed.starts_with("*/")
                {
                    byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
                    continue;
                }

                // Skip string directives like "use strict"
                if trimmed.starts_with('"') || trimmed.starts_with('\'') {
                    let without_semi = trimmed.trim_end_matches(';');
                    if (without_semi.starts_with('"') && without_semi.ends_with('"'))
                        || (without_semi.starts_with('\'') && without_semi.ends_with('\''))
                    {
                        byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
                        continue;
                    }
                }

                if trimmed.starts_with("import ") || trimmed.starts_with("import{") {
                    if found_non_import {
                        let start = u32::try_from(byte_offset).unwrap_or(0);
                        let end =
                            u32::try_from(byte_offset.saturating_add(line.len())).unwrap_or(start);
                        results.push((start, end));
                    }
                } else if trimmed.starts_with("export ") && trimmed.contains(" from ") {
                    // Re-export — treat like import
                    if found_non_import {
                        let start = u32::try_from(byte_offset).unwrap_or(0);
                        let end =
                            u32::try_from(byte_offset.saturating_add(line.len())).unwrap_or(start);
                        results.push((start, end));
                    }
                } else {
                    found_non_import = true;
                }

                byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
            }

            results
        };

        for (start, end) in findings {
            ctx.report(Diagnostic {
                rule_name: "import/first".to_owned(),
                message: "Import declarations must appear before other statements".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Move this import to the top of the file".to_owned()),
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(First)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_import_after_statement() {
        let source = "const x = 1;\nimport foo from 'foo';";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "import after statement should be flagged");
    }

    #[test]
    fn test_allows_imports_first() {
        let source = "import foo from 'foo';\nimport bar from 'bar';\nconst x = 1;";
        let diags = lint(source);
        assert!(diags.is_empty(), "imports before statements should be fine");
    }

    #[test]
    fn test_allows_directive_before_import() {
        let source = "\"use strict\";\nimport foo from 'foo';";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "directive before import should not be flagged"
        );
    }
}
