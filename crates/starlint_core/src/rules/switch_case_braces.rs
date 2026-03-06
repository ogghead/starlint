//! Rule: `switch-case-braces`
//!
//! Enforces braces around `switch` case bodies. Without braces, variables
//! declared in one case are visible in all subsequent cases, which is a
//! common source of scope-related bugs. Wrapping each case body in a block
//! creates a lexical scope boundary.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `switch` case clauses whose body is not wrapped in a `BlockStatement`.
#[derive(Debug)]
pub struct SwitchCaseBraces;

impl NativeRule for SwitchCaseBraces {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "switch-case-braces".to_owned(),
            description: "Enforce braces around switch case bodies".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SwitchCase])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SwitchCase(case) = kind else {
            return;
        };

        // Skip empty cases (fall-through grouping like `case 1: case 2:`)
        if case.consequent.is_empty() {
            return;
        }

        // Check if the entire consequent is a single BlockStatement
        let is_wrapped_in_block = case.consequent.len() == 1
            && matches!(case.consequent.first(), Some(Statement::BlockStatement(_)));

        if !is_wrapped_in_block {
            let case_span = Span::new(case.span.start, case.span.end);
            // Wrap the consequent statements in braces
            // Get the span from first to last statement
            let first_start = case.consequent.first().map_or(0, |s| s.span().start);
            let last_end = case.consequent.last().map_or(0, |s| s.span().end);
            let body_span = Span::new(first_start, last_end);
            let source = ctx.source_text();
            let body_text = source
                .get(
                    usize::try_from(first_start).unwrap_or(0)
                        ..usize::try_from(last_end).unwrap_or(0),
                )
                .unwrap_or("");
            let replacement = format!("{{ {body_text} }}");
            ctx.report(Diagnostic {
                rule_name: "switch-case-braces".to_owned(),
                message: "Switch case body should be wrapped in braces".to_owned(),
                span: case_span,
                severity: Severity::Warning,
                help: Some("Wrap the case body in braces".to_owned()),
                fix: Some(Fix {
                    message: "Wrap in braces".to_owned(),
                    edits: vec![Edit {
                        span: body_span,
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SwitchCaseBraces)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_case_without_braces() {
        let diags = lint("switch(x) { case 1: foo(); break; }");
        assert_eq!(diags.len(), 1, "case without braces should be flagged");
    }

    #[test]
    fn test_allows_case_with_braces() {
        let diags = lint("switch(x) { case 1: { foo(); break; } }");
        assert!(diags.is_empty(), "case with braces should not be flagged");
    }

    #[test]
    fn test_flags_break_only_without_braces() {
        let diags = lint("switch(x) { case 1: break; }");
        assert_eq!(
            diags.len(),
            1,
            "case with just break (no braces) should be flagged"
        );
    }

    #[test]
    fn test_flags_default_without_braces() {
        let diags = lint("switch(x) { default: break; }");
        assert_eq!(
            diags.len(),
            1,
            "default case without braces should be flagged"
        );
    }

    #[test]
    fn test_allows_default_with_braces() {
        let diags = lint("switch(x) { default: { break; } }");
        assert!(
            diags.is_empty(),
            "default case with braces should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_case_fallthrough() {
        let diags = lint("switch(x) { case 1: case 2: { foo(); break; } }");
        assert!(
            diags.is_empty(),
            "empty case for fallthrough grouping should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_cases_without_braces() {
        let diags = lint("switch(x) { case 1: foo(); break; case 2: bar(); break; }");
        assert_eq!(
            diags.len(),
            2,
            "both cases without braces should be flagged"
        );
    }
}
