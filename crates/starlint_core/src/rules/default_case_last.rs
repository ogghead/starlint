//! Rule: `default-case-last`
//!
//! Require the `default` case in switch statements to be the last case.
//! Placing the default case in the middle of a switch makes it harder to
//! understand the flow, because you have to mentally skip over it when
//! reading the subsequent cases.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `default` cases that are not the last case in a switch statement.
#[derive(Debug)]
pub struct DefaultCaseLast;

impl NativeRule for DefaultCaseLast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "default-case-last".to_owned(),
            description: "Require `default` case to be last in `switch` statements".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SwitchStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SwitchStatement(switch) = kind else {
            return;
        };

        let cases = &switch.cases;
        let case_count = cases.len();

        // No cases or only one case — nothing to flag
        if case_count <= 1 {
            return;
        }

        for (i, case) in cases.iter().enumerate() {
            // The default case has no test expression
            let is_default = case.test.is_none();
            let is_last = i.saturating_add(1) >= case_count;

            if is_default && !is_last {
                ctx.report(Diagnostic {
                    rule_name: "default-case-last".to_owned(),
                    message: "The `default` case should be the last case in a `switch` statement"
                        .to_owned(),
                    span: Span::new(case.span.start, case.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
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

    /// Helper to lint source code with the `DefaultCaseLast` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DefaultCaseLast)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_default_not_last() {
        let diags = lint("switch(x) { case 1: break; default: break; case 2: break; }");
        assert_eq!(
            diags.len(),
            1,
            "default case not at the end should be flagged"
        );
    }

    #[test]
    fn test_flags_default_first() {
        let diags = lint("switch(x) { default: break; case 1: break; case 2: break; }");
        assert_eq!(
            diags.len(),
            1,
            "default case at the beginning should be flagged"
        );
    }

    #[test]
    fn test_allows_default_last() {
        let diags = lint("switch(x) { case 1: break; default: break; }");
        assert!(
            diags.is_empty(),
            "default case at the end should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_default() {
        let diags = lint("switch(x) { case 1: break; case 2: break; }");
        assert!(
            diags.is_empty(),
            "switch without default should not be flagged"
        );
    }

    #[test]
    fn test_allows_only_default() {
        let diags = lint("switch(x) { default: break; }");
        assert!(
            diags.is_empty(),
            "switch with only default case should not be flagged"
        );
    }

    #[test]
    fn test_allows_default_last_of_many() {
        let diags =
            lint("switch(x) { case 1: break; case 2: break; case 3: break; default: break; }");
        assert!(
            diags.is_empty(),
            "default as last of many cases should not be flagged"
        );
    }
}
