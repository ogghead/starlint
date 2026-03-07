//! Rule: `typescript/triple-slash-reference`
//!
//! Disallow `/// <reference ... />` directives. Triple-slash reference
//! directives are an older mechanism for declaring dependencies between
//! files. Modern TypeScript projects should use `import` statements or
//! `tsconfig.json` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `/// <reference ... />` directives in source text.
#[derive(Debug)]
pub struct TripleSlashReference;

impl LintRule for TripleSlashReference {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/triple-slash-reference".to_owned(),
            description: "Disallow `/// <reference ... />` directives".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_triple_slash_references(source);

        for (start, end, line_start, line_end) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/triple-slash-reference".to_owned(),
                message: "Do not use `/// <reference />` directives — use `import` or `tsconfig.json` instead".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Remove the triple-slash reference directive".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Delete the triple-slash reference line".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(line_start, line_end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// The prefix that identifies a triple-slash reference directive.
const REFERENCE_PREFIX: &str = "/// <reference";

/// Scan source text for `/// <reference ... />` directives.
///
/// Returns a list of `(start_offset, end_offset, line_start, line_end)` tuples
/// for each line that starts with `/// <reference`. The `line_start`/`line_end`
/// spans include leading whitespace and the trailing newline for deletion.
fn find_triple_slash_references(source: &str) -> Vec<(u32, u32, u32, u32)> {
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
            // For deletion: include from the start of the line to the newline
            let line_start_u32 = u32::try_from(offset).unwrap_or(0);
            let line_end_u32 = u32::try_from(
                offset
                    .saturating_add(line.len())
                    .saturating_add(1)
                    .min(source.len()),
            )
            .unwrap_or(end_u32);
            results.push((start_u32, end_u32, line_start_u32, line_end_u32));
        }
        // Account for the line content plus the newline character
        offset = offset.saturating_add(line.len()).saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(TripleSlashReference)];
        lint_source(source, "test.js", &rules)
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
