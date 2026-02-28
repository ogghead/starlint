//! Rule: `import/max-dependencies`
//!
//! Limit the number of dependencies a module can have. Modules with many
//! imports are harder to understand and may indicate a need for refactoring.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default maximum number of dependencies.
const DEFAULT_MAX: usize = 10;

/// Flags modules that import from too many distinct sources.
#[derive(Debug)]
pub struct MaxDependencies {
    /// Maximum number of import sources allowed.
    max: usize,
}

impl Default for MaxDependencies {
    fn default() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl MaxDependencies {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl NativeRule for MaxDependencies {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/max-dependencies".to_owned(),
            description: "Limit the number of dependencies a module can have".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            self.max = usize::try_from(n).unwrap_or(DEFAULT_MAX);
        }
        Ok(())
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let (dep_count, source_len) = {
            let source = ctx.source_text();
            let mut sources: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut search_start: usize = 0;

            while let Some(pos) = source.get(search_start..).and_then(|s| s.find("import ")) {
                let abs_pos = search_start.saturating_add(pos);

                // Check word boundary
                let is_start = abs_pos == 0
                    || source
                        .as_bytes()
                        .get(abs_pos.saturating_sub(1))
                        .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

                if is_start {
                    let line_end = source
                        .get(abs_pos..)
                        .and_then(|s| s.find('\n'))
                        .map_or(source.len(), |p| abs_pos.saturating_add(p));

                    if let Some(line) = source.get(abs_pos..line_end) {
                        if let Some(from_source) = extract_import_source(line) {
                            sources.insert(from_source.to_owned());
                        }
                    }
                }

                search_start = abs_pos.saturating_add("import ".len());
            }

            (sources.len(), u32::try_from(source.len()).unwrap_or(0))
        };

        if dep_count > self.max {
            ctx.report_warning(
                "import/max-dependencies",
                &format!(
                    "Module has too many dependencies ({dep_count}). Maximum allowed is {}",
                    self.max,
                ),
                Span::new(0, source_len),
            );
        }
    }
}

/// Extract the module source string from an import line.
fn extract_import_source(line: &str) -> Option<&str> {
    // Look for the string after 'from' or a direct import 'xxx'
    let from_idx = line.find(" from ")?;
    let after_from = line.get(from_idx.saturating_add(6)..)?;
    extract_quoted_string(after_from)
}

/// Extract a quoted string value from the start of a trimmed string.
fn extract_quoted_string(s: &str) -> Option<&str> {
    let trimmed = s.trim();
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint_with_max(source: &str, max: usize) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxDependencies { max })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_too_many_dependencies() {
        let source = "import a from 'a';\nimport b from 'b';\nimport c from 'c';";
        let diags = lint_with_max(source, 2);
        assert_eq!(
            diags.len(),
            1,
            "module with 3 dependencies (max 2) should be flagged"
        );
    }

    #[test]
    fn test_allows_within_limit() {
        let source = "import a from 'a';\nimport b from 'b';";
        let diags = lint_with_max(source, 5);
        assert!(
            diags.is_empty(),
            "module within limit should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_imports() {
        let source = "const x = 1;";
        let diags = lint_with_max(source, 1);
        assert!(diags.is_empty(), "module with no imports should be fine");
    }
}
