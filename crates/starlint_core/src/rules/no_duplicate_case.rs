//! Rule: `no-duplicate-case`
//!
//! Disallow duplicate case labels in `switch` statements. If a `switch`
//! statement has duplicate case expressions, the second case will never
//! be reached (the first matching case always wins).

use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `switch` statements with duplicate `case` labels.
#[derive(Debug)]
pub struct NoDuplicateCase;

impl NativeRule for NoDuplicateCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-duplicate-case".to_owned(),
            description: "Disallow duplicate case labels".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SwitchStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SwitchStatement(switch) = kind else {
            return;
        };

        let mut seen = HashSet::new();

        for case in &switch.cases {
            let Some(test) = &case.test else {
                // `default:` has no test expression
                continue;
            };

            // Use the source text of the test expression as the key for
            // duplicate detection. This handles identifiers, literals, and
            // simple expressions. More complex equivalence checking (e.g.
            // `1+2` vs `3`) is intentionally not done.
            let test_span = test.span();
            let start = usize::try_from(test_span.start).unwrap_or(0);
            let end = usize::try_from(test_span.end).unwrap_or(0);
            let Some(source_slice) = ctx.source_text().get(start..end) else {
                continue;
            };
            let key = source_slice.to_owned();

            if !seen.insert(key.clone()) {
                // Fix: delete the entire duplicate case clause
                let fix = Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Remove duplicate case clause".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(case.span.start, case.span.end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                });
                ctx.report(Diagnostic {
                    rule_name: "no-duplicate-case".to_owned(),
                    message: format!("Duplicate case label `{key}`"),
                    span: Span::new(test_span.start, test_span.end),
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDuplicateCase)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_case() {
        let diags = lint("switch(x) { case 1: break; case 1: break; }");
        assert_eq!(diags.len(), 1, "duplicate case 1 should be flagged");
    }

    #[test]
    fn test_flags_duplicate_string_case() {
        let diags = lint(r#"switch(x) { case "a": break; case "a": break; }"#);
        assert_eq!(diags.len(), 1, "duplicate string case should be flagged");
    }

    #[test]
    fn test_flags_multiple_duplicates() {
        let diags =
            lint("switch(x) { case 1: break; case 2: break; case 1: break; case 2: break; }");
        assert_eq!(
            diags.len(),
            2,
            "two pairs of duplicates should produce two diagnostics"
        );
    }

    #[test]
    fn test_allows_unique_cases() {
        let diags = lint("switch(x) { case 1: break; case 2: break; case 3: break; }");
        assert!(diags.is_empty(), "unique cases should not be flagged");
    }

    #[test]
    fn test_allows_default_case() {
        let diags = lint("switch(x) { case 1: break; default: break; }");
        assert!(diags.is_empty(), "default case should not be flagged");
    }

    #[test]
    fn test_allows_duplicate_identifier_different_names() {
        let diags = lint("switch(x) { case a: break; case b: break; }");
        assert!(
            diags.is_empty(),
            "different identifiers should not be flagged"
        );
    }
}
