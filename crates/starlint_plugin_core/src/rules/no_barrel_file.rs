//! Rule: `no-barrel-file` (oxc)
//!
//! Flag barrel files (index files that only re-export from other modules).
//! Barrel files hurt tree-shaking because bundlers must evaluate the entire
//! re-export chain to determine what is actually used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// File name suffixes that qualify as barrel files.
const BARREL_SUFFIXES: &[&str] = &[
    "index.js",
    "index.ts",
    "index.mjs",
    "index.mts",
    "index.jsx",
    "index.tsx",
];

/// Flags files that are barrel files (only re-export from other modules).
#[derive(Debug)]
pub struct NoBarrelFile;

/// Check whether a file path ends with one of the barrel file suffixes.
fn is_index_file(file_path: &std::path::Path) -> bool {
    let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    BARREL_SUFFIXES.contains(&file_name)
}

/// Check whether source text consists entirely of re-export statements.
fn is_barrel_source(source: &str) -> bool {
    let trimmed_source = source.trim();

    // Empty files are not barrel files
    if trimmed_source.is_empty() {
        return false;
    }

    let mut has_statements = false;

    for line in trimmed_source.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }

        has_statements = true;

        // Check for re-export patterns:
        // `export { ... } from '...';` or `export * from '...';`
        let is_export_from = trimmed.starts_with("export") && trimmed.contains("from");
        if !is_export_from {
            return false;
        }
    }

    has_statements
}

impl LintRule for NoBarrelFile {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-barrel-file".to_owned(),
            description: "Disallow barrel files (index files that only re-export)".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        // Only flag index files
        if !is_index_file(ctx.file_path()) {
            return;
        }

        let is_barrel = is_barrel_source(ctx.source_text());

        if is_barrel {
            let source_len = u32::try_from(ctx.source_text().len()).unwrap_or(0);
            ctx.report(Diagnostic {
                rule_name: "no-barrel-file".to_owned(),
                message:
                    "Barrel files hurt tree-shaking. Import directly from the source module instead"
                        .to_owned(),
                span: Span::new(0, source_len),
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
    fn lint_with_path(
        source: &str,
        path: &str,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoBarrelFile)];
        lint_source(source, path, &rules)
    }

    #[test]
    fn test_flags_barrel_file() {
        let source = "export { foo } from './foo';\nexport { bar } from './bar';";
        let diags = lint_with_path(source, "index.js");
        assert_eq!(diags.len(), 1, "barrel file should be flagged");
    }

    #[test]
    fn test_flags_star_reexport_barrel() {
        let source = "export * from './foo';\nexport * from './bar';";
        let diags = lint_with_path(source, "index.ts");
        assert_eq!(diags.len(), 1, "star re-export barrel should be flagged");
    }

    #[test]
    fn test_allows_file_with_own_code() {
        let source = "export function foo() {}";
        let diags = lint_with_path(source, "index.js");
        assert!(
            diags.is_empty(),
            "file with own declarations should not be flagged"
        );
    }

    #[test]
    fn test_allows_mixed_file() {
        let source = "const x = 1;\nexport { x };";
        let diags = lint_with_path(source, "index.js");
        assert!(
            diags.is_empty(),
            "file with own code and exports should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_index_file() {
        let source = "export { foo } from './foo';\nexport { bar } from './bar';";
        let diags = lint_with_path(source, "utils.js");
        assert!(
            diags.is_empty(),
            "non-index file should not be flagged even if all re-exports"
        );
    }

    #[test]
    fn test_allows_empty_index() {
        let diags = lint_with_path("", "index.js");
        assert!(diags.is_empty(), "empty index file should not be flagged");
    }
}
