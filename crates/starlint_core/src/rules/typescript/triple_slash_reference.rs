//! Rule: `typescript/triple-slash-reference`
//!
//! Disallow `/// <reference ... />` directives. Triple-slash reference
//! directives are an older mechanism for declaring dependencies between
//! files. Modern TypeScript projects should use `import` statements or
//! `tsconfig.json` instead.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `/// <reference ... />` directives in source text.
#[derive(Debug)]
pub struct TripleSlashReference;

impl NativeRule for TripleSlashReference {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/triple-slash-reference".to_owned(),
            description: "Disallow `/// <reference ... />` directives".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings = find_triple_slash_references(ctx.source_text());

        for (start, end) in findings {
            ctx.report_warning(
                "typescript/triple-slash-reference",
                "Do not use `/// <reference />` directives — use `import` or `tsconfig.json` instead",
                Span::new(start, end),
            );
        }
    }
}

/// The prefix that identifies a triple-slash reference directive.
const REFERENCE_PREFIX: &str = "/// <reference";

/// Scan source text for `/// <reference ... />` directives.
///
/// Returns a list of `(start_offset, end_offset)` tuples for each line
/// that starts with `/// <reference`.
fn find_triple_slash_references(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let mut offset: usize = 0;

    for line in source.split('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with(REFERENCE_PREFIX) {
            let leading_whitespace = line.len().saturating_sub(trimmed.len());
            let start = offset.saturating_add(leading_whitespace);
            let end = offset.saturating_add(line.len());
            let start_u32 = u32::try_from(start).unwrap_or(0);
            let end_u32 = u32::try_from(end).unwrap_or(start_u32);
            results.push((start_u32, end_u32));
        }
        // Account for the line content plus the newline character
        offset = offset.saturating_add(line.len()).saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(TripleSlashReference)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reference_path() {
        let diags = lint(r#"/// <reference path="foo" />"#);
        assert_eq!(
            diags.len(),
            1,
            "`/// <reference path>` directive should be flagged"
        );
    }

    #[test]
    fn test_flags_reference_types() {
        let diags = lint(r#"/// <reference types="node" />"#);
        assert_eq!(
            diags.len(),
            1,
            "`/// <reference types>` directive should be flagged"
        );
    }

    #[test]
    fn test_allows_regular_comment() {
        let diags = lint("// regular comment\nlet x = 1;");
        assert!(diags.is_empty(), "regular comments should not be flagged");
    }

    #[test]
    fn test_allows_regular_triple_slash() {
        let diags = lint("/// regular triple-slash comment\nlet x = 1;");
        assert!(
            diags.is_empty(),
            "triple-slash comments without `<reference` should not be flagged"
        );
    }
}
