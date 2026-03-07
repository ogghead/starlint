//! Rule: `typescript/no-unsafe-declaration-merging`
//!
//! Disallow unsafe declaration merging. When a class and an interface share the
//! same name in the same scope, TypeScript merges their declarations. This can
//! lead to unexpected runtime behavior because the interface adds type-level
//! members that the class does not actually implement at runtime.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unsafe-declaration-merging";

/// Flags files where a class and an interface share the same name.
///
/// Uses a text-based approach via `run_once()` to scan for `class X` and
/// `interface X` patterns, flagging any name that appears in both.
#[derive(Debug)]
pub struct NoUnsafeDeclarationMerging;

impl LintRule for NoUnsafeDeclarationMerging {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unsafe declaration merging between classes and interfaces"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        // Collect all data from source text first, then report.
        // This avoids holding an immutable borrow on ctx while reporting.
        let findings: Vec<(String, u32, u32)> = {
            let source = ctx.source_text();
            let class_names = collect_declaration_names(source, "class ");
            let interface_decls = collect_declarations_with_positions(source, "interface ");

            interface_decls
                .into_iter()
                .filter(|(name, _, _)| class_names.iter().any(|cn| cn == name))
                .map(|(name, start, end)| (name.to_owned(), start, end))
                .collect()
        };

        for (name, start, end) in &findings {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Unsafe declaration merging — interface `{name}` merges with a class of the same name"
                ),
                span: Span::new(*start, *end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Collect identifier names following a keyword (e.g. `class ` or `interface `).
///
/// Returns a list of names found in the source text. This is a simplified
/// text-based scan — it does not handle all edge cases (e.g. names inside
/// strings or comments) but works well for typical TypeScript source files.
fn collect_declaration_names<'a>(source: &'a str, keyword: &str) -> Vec<&'a str> {
    let mut names = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(keyword)) {
        let absolute_pos = search_from.saturating_add(pos);
        let name_start = absolute_pos.saturating_add(keyword.len());

        if let Some(name) = extract_identifier(source, name_start) {
            names.push(name);
        }

        search_from = name_start;
    }

    names
}

/// Collect declarations with their positions for reporting.
///
/// Returns a list of `(name, keyword_start, name_end)` tuples.
fn collect_declarations_with_positions<'a>(
    source: &'a str,
    keyword: &str,
) -> Vec<(&'a str, u32, u32)> {
    let mut results = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find(keyword)) {
        let absolute_pos = search_from.saturating_add(pos);
        let name_start = absolute_pos.saturating_add(keyword.len());

        if let Some(name) = extract_identifier(source, name_start) {
            let name_end = name_start.saturating_add(name.len());
            let start_u32 = u32::try_from(absolute_pos).unwrap_or(0);
            let end_u32 = u32::try_from(name_end).unwrap_or(start_u32);
            results.push((name, start_u32, end_u32));
        }

        search_from = name_start;
    }

    results
}

/// Extract an identifier starting at `pos` in `source`.
///
/// Returns the identifier slice if one is found at the given position.
fn extract_identifier(source: &str, pos: usize) -> Option<&str> {
    let remaining = source.get(pos..)?;
    let end = remaining
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(remaining.len());

    if end == 0 {
        return None;
    }

    remaining.get(..end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeDeclarationMerging)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_class_and_interface_same_name() {
        let diags = lint("class Foo {}\ninterface Foo {}");
        assert_eq!(
            diags.len(),
            1,
            "class and interface with same name should be flagged"
        );
    }

    #[test]
    fn test_flags_interface_before_class() {
        let diags = lint("interface Bar {}\nclass Bar {}");
        assert_eq!(
            diags.len(),
            1,
            "interface before class with same name should be flagged"
        );
    }

    #[test]
    fn test_allows_class_only() {
        let diags = lint("class Foo {}");
        assert!(diags.is_empty(), "class alone should not be flagged");
    }

    #[test]
    fn test_allows_interface_only() {
        let diags = lint("interface Foo {}");
        assert!(diags.is_empty(), "interface alone should not be flagged");
    }

    #[test]
    fn test_allows_different_names() {
        let diags = lint("class Foo {}\ninterface Bar {}");
        assert!(
            diags.is_empty(),
            "class and interface with different names should not be flagged"
        );
    }
}
