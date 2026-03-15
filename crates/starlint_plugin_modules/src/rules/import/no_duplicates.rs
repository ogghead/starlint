//! Rule: `import/no-duplicates`
//!
//! Report duplicate imports from the same module. Multiple import statements
//! from the same source should be merged into a single import for clarity
//! and to avoid confusion about what is imported.

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags duplicate import declarations for the same module source.
#[derive(Debug)]
pub struct NoDuplicates;

impl LintRule for NoDuplicates {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-duplicates".to_owned(),
            description: "Report duplicate imports from the same module".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let import_sources: HashMap<String, Vec<(u32, u32)>> = {
            let source = ctx.source_text();
            let mut sources: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
            let mut byte_offset: usize = 0;

            for line in source.lines() {
                let trimmed = line.trim();

                if trimmed.starts_with("import ") || trimmed.starts_with("import{") {
                    if let Some(module_source) = extract_import_source_from_line(trimmed) {
                        let start = u32::try_from(byte_offset).unwrap_or(0);
                        let end =
                            u32::try_from(byte_offset.saturating_add(line.len())).unwrap_or(start);
                        sources
                            .entry(module_source.to_owned())
                            .or_default()
                            .push((start, end));
                    }
                }

                byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
            }

            sources
        };

        // Report duplicates (skip the first occurrence, flag subsequent ones).
        // Each duplicate gets a fix to merge it into the first import.
        for (module_source, positions) in &import_sources {
            if positions.len() > 1 {
                let first = positions.first().copied();
                for &(start, end) in positions.iter().skip(1) {
                    let fix = first.and_then(|(first_start, first_end)| {
                        let edits = fix_utils::merge_import_edits(
                            ctx.source_text(),
                            Span::new(first_start, first_end),
                            Span::new(start, end),
                        );
                        FixBuilder::new("Merge into first import", FixKind::DangerousFix)
                            .edits(edits)
                            .build()
                    });
                    ctx.report(Diagnostic {
                        rule_name: "import/no-duplicates".to_owned(),
                        message: format!(
                            "'{module_source}' is imported multiple times; merge into a single import"
                        ),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: Some("Merge duplicate imports into one statement".to_owned()),
                        fix,
                        labels: vec![],
                    });
                }
            }
        }
    }
}

/// Extract the module source string from an import line.
fn extract_import_source_from_line(line: &str) -> Option<&str> {
    // Look for `from '...'` or `from "..."`
    if let Some(from_idx) = line.find(" from ") {
        let after_from = line.get(from_idx.saturating_add(6)..)?;
        return extract_quoted(after_from);
    }

    // Side-effect import: `import 'module'` or `import "module"`
    let after_import = line.strip_prefix("import ")?.trim();
    if after_import.starts_with('\'') || after_import.starts_with('"') {
        return extract_quoted(after_import);
    }

    None
}

/// Extract a quoted string value.
fn extract_quoted(s: &str) -> Option<&str> {
    let trimmed = s.trim().trim_end_matches(';').trim();
    let quote = trimmed.as_bytes().first()?;
    if *quote != b'\'' && *quote != b'"' {
        return None;
    }
    let rest = trimmed.get(1..)?;
    let end = rest.find(char::from(*quote))?;
    rest.get(..end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_rule_framework::fix::apply_fixes;
    starlint_rule_framework::lint_rule_test!(NoDuplicates);

    #[test]
    fn test_flags_duplicate_imports() {
        let source = "import { foo } from 'mod';\nimport { bar } from 'mod';";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "duplicate import from same module should be flagged"
        );
    }

    #[test]
    fn test_allows_unique_imports() {
        let source = "import { foo } from 'mod1';\nimport { bar } from 'mod2';";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "imports from different modules should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_import() {
        let source = "import { foo, bar } from 'mod';";
        let diags = lint(source);
        assert!(diags.is_empty(), "single import should not be flagged");
    }

    #[test]
    fn test_fix_merges_duplicate_imports() {
        let source = "import { foo } from 'mod';\nimport { bar } from 'mod';\n";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "should flag duplicate");
        assert!(
            diags.first().is_some_and(|d| d.fix.is_some()),
            "should provide a fix"
        );
        let fixed = apply_fixes(source, &diags);
        assert!(
            fixed.contains("foo") && fixed.contains("bar"),
            "merged import should contain both specifiers"
        );
        assert_eq!(
            fixed.matches("import").count(),
            1,
            "should have exactly one import statement after merge"
        );
    }

    #[test]
    fn test_fix_not_provided_for_default_imports() {
        let source = "import foo from 'mod';\nimport bar from 'mod';\n";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "should flag duplicate");
        assert!(
            diags.first().is_some_and(|d| d.fix.is_none()),
            "default imports should not have a merge fix"
        );
    }
}
