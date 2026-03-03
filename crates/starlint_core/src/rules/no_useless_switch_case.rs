//! Rule: `no-useless-switch-case` (unicorn)
//!
//! Disallow useless case in switch statements. A switch with only a
//! default case, or a case that has the same body as the default case
//! (falling through), is unnecessary complexity.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags switch statements where all cases simply fall through to default.
#[derive(Debug)]
pub struct NoUselessSwitchCase;

impl NativeRule for NoUselessSwitchCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-switch-case".to_owned(),
            description: "Disallow useless case in switch statements".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SwitchStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SwitchStatement(switch) = kind else {
            return;
        };

        // Find the default case
        let has_default = switch.cases.iter().any(|c| c.test.is_none());
        if !has_default {
            return;
        }

        // Check for cases that fall through to default:
        // A case is "useless" if it has an empty body and the next case is default
        // (or it IS the case right before default with no body)
        let cases = &switch.cases;
        let case_count = cases.len();

        for (i, case) in cases.iter().enumerate() {
            // Skip the default case itself
            if case.test.is_none() {
                continue;
            }

            // If this case has an empty consequent and the next case is default,
            // it's useless — it just falls through to default
            if case.consequent.is_empty() {
                // Check if the next case is default
                let next_is_default = cases
                    .get(i.saturating_add(1))
                    .is_some_and(|next| next.test.is_none());

                if next_is_default {
                    ctx.report_warning(
                        "no-useless-switch-case",
                        "Useless case — falls through to default",
                        Span::new(case.span.start, case.span.end),
                    );
                }
            }

            // If it's the only non-default case and has the same sole
            // body as default, flag it. Check: switch has exactly 2 cases
            // (one test case + default) and both have the same body text.
            if case_count == 2 && !case.consequent.is_empty() {
                let default_case = cases.iter().find(|c| c.test.is_none());
                if let Some(dc) = default_case {
                    if !dc.consequent.is_empty() {
                        let source = ctx.source_text();
                        let case_body = get_consequent_text(source, case);
                        let default_body = get_consequent_text(source, dc);
                        if case_body == default_body && !case_body.is_empty() {
                            ctx.report_warning(
                                "no-useless-switch-case",
                                "Useless case — has the same body as default",
                                Span::new(case.span.start, case.span.end),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Extract the text of a switch case's consequent from source.
fn get_consequent_text<'a>(source: &'a str, case: &oxc_ast::ast::SwitchCase<'_>) -> &'a str {
    let Some(first) = case.consequent.first() else {
        return "";
    };
    let Some(last) = case.consequent.last() else {
        return "";
    };
    let start = usize::try_from(first.span().start).unwrap_or(0);
    let end = usize::try_from(last.span().end)
        .unwrap_or(0)
        .min(source.len());
    source.get(start..end).unwrap_or("").trim()
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessSwitchCase)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_case_before_default() {
        let diags = lint("switch (x) { case 1: default: break; }");
        assert_eq!(
            diags.len(),
            1,
            "empty case falling to default should be flagged"
        );
    }

    #[test]
    fn test_allows_case_with_body() {
        let diags = lint("switch (x) { case 1: foo(); break; default: break; }");
        assert!(
            diags.is_empty(),
            "case with its own body should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_default() {
        let diags = lint("switch (x) { case 1: break; case 2: break; }");
        assert!(
            diags.is_empty(),
            "switch without default should not be flagged"
        );
    }

    #[test]
    fn test_allows_separate_behaviors() {
        let diags = lint(
            "switch (x) { case 1: foo(); break; case 2: bar(); break; default: baz(); break; }",
        );
        assert!(
            diags.is_empty(),
            "cases with different behaviors should not be flagged"
        );
    }
}
