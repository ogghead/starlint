//! Rule: `typescript/consistent-type-imports`
//!
//! Prefer `import type` for type-only imports instead of mixing inline `type`
//! qualifiers within regular import statements. When an import statement
//! contains `import { type Foo, type Bar }`, it should be rewritten as
//! `import type { Foo, Bar }` for clarity and consistency. This makes it
//! immediately clear at the statement level that the import is type-only,
//! which helps bundlers and readers alike.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags import statements that use inline `type` qualifiers when all
/// imported specifiers are type-only.
#[derive(Debug)]
pub struct ConsistentTypeImports;

impl NativeRule for ConsistentTypeImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/consistent-type-imports".to_owned(),
            description: "Prefer `import type` over inline `type` qualifiers in imports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_inline_type_imports(source);

        for (start, end) in findings {
            ctx.report_warning(
                "typescript/consistent-type-imports",
                "Use `import type { ... }` instead of `import { type ... }`",
                Span::new(start, end),
            );
        }
    }
}

/// Find import statements that use inline `type` qualifiers for all specifiers.
///
/// Detects patterns like `import { type Foo, type Bar } from "mod"` where
/// every specifier has the `type` keyword, suggesting the entire import
/// should be `import type { Foo, Bar } from "mod"` instead.
///
/// Returns `(start, end)` byte offsets for each flagged import statement.
fn find_inline_type_imports(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();

    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Must be a regular import (not already `import type`)
        if !trimmed.starts_with("import ") || trimmed.starts_with("import type ") {
            continue;
        }

        // Must have braces (named imports)
        let Some(brace_start) = trimmed.find('{') else {
            continue;
        };
        let Some(brace_end) = trimmed.find('}') else {
            continue;
        };

        if brace_end <= brace_start {
            continue;
        }

        let specifiers_str = trimmed
            .get(brace_start.saturating_add(1)..brace_end)
            .unwrap_or("");

        // Split on commas and check each specifier
        let specifiers: Vec<&str> = specifiers_str
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        if specifiers.is_empty() {
            continue;
        }

        // Check if ALL specifiers have the `type` keyword
        let all_type = specifiers.iter().all(|s| s.starts_with("type "));
        let has_any_type = specifiers.iter().any(|s| s.starts_with("type "));

        if all_type && has_any_type {
            // Calculate byte offset for this line
            let line_start = line_byte_offset(source, line_idx);
            let line_end = line_start.saturating_add(line.len());
            let start = u32::try_from(line_start).unwrap_or(0);
            let end = u32::try_from(line_end).unwrap_or(start);
            results.push((start, end));
        }
    }

    results
}

/// Calculate the byte offset of the start of line `line_idx` in `source`.
fn line_byte_offset(source: &str, line_idx: usize) -> usize {
    let mut offset: usize = 0;
    for (i, line) in source.lines().enumerate() {
        if i == line_idx {
            return offset;
        }
        // +1 for the newline character
        offset = offset.saturating_add(line.len()).saturating_add(1);
    }
    offset
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentTypeImports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_all_inline_type_imports() {
        let source = r#"import { type Foo, type Bar } from "mod";"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "import with all inline type specifiers should be flagged"
        );
    }

    #[test]
    fn test_flags_single_inline_type_import() {
        let source = r#"import { type Foo } from "mod";"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "import with single inline type specifier should be flagged"
        );
    }

    #[test]
    fn test_allows_import_type_syntax() {
        let source = r#"import type { Foo, Bar } from "mod";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`import type` syntax should not be flagged"
        );
    }

    #[test]
    fn test_allows_mixed_imports() {
        let source = r#"import { type Foo, bar } from "mod";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "mixed type and value imports should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_imports() {
        let source = r#"import { Foo, Bar } from "mod";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "regular value imports should not be flagged"
        );
    }
}
