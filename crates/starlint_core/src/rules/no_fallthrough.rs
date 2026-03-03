//! Rule: `no-fallthrough`
//!
//! Disallow fallthrough of `case` statements in `switch`. Unintentional
//! fallthrough is a common source of bugs. Cases that intentionally fall
//! through should have a `// falls through` comment (not yet supported).
//!
//! Note: This is a basic implementation that does not yet check for
//! `// falls through` or `// no break` comments. A full implementation
//! requires comment extraction infrastructure.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags switch case fallthrough (cases without `break`, `return`, or `throw`).
#[derive(Debug)]
pub struct NoFallthrough;

impl NativeRule for NoFallthrough {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-fallthrough".to_owned(),
            description: "Disallow fallthrough of `case` statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        let cases = &switch.cases;
        let case_count = cases.len();

        // Collect fallthrough cases first to avoid borrow issues
        let mut fallthrough_spans: Vec<Span> = Vec::new();

        for (i, case) in cases.iter().enumerate() {
            // Skip the last case — no fallthrough possible
            let is_last = i.saturating_add(1) >= case_count;
            if is_last {
                continue;
            }

            // Empty cases are intentional fallthrough (grouping)
            if case.consequent.is_empty() {
                continue;
            }

            // Check if the case ends with a terminator
            if !ends_with_terminator(&case.consequent) {
                fallthrough_spans.push(Span::new(case.span.start, case.span.end));
            }
        }

        for span in fallthrough_spans {
            ctx.report_error(
                "no-fallthrough",
                "Expected a `break` statement before falling through to the next case",
                span,
            );
        }
    }
}

/// Check if a list of statements ends with a control flow terminator.
fn ends_with_terminator(stmts: &[Statement<'_>]) -> bool {
    let Some(last) = stmts.last() else {
        return false;
    };

    match last {
        Statement::ReturnStatement(_)
        | Statement::ThrowStatement(_)
        | Statement::BreakStatement(_)
        | Statement::ContinueStatement(_) => true,
        Statement::BlockStatement(block) => ends_with_terminator(&block.body),
        Statement::IfStatement(if_stmt) => {
            // Both branches must terminate
            let consequent_terminates = statement_terminates(&if_stmt.consequent);
            let alternate_terminates = if_stmt
                .alternate
                .as_ref()
                .is_some_and(|alt| statement_terminates(alt));
            consequent_terminates && alternate_terminates
        }
        _ => false,
    }
}

/// Check if a single statement terminates control flow.
fn statement_terminates(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(_)
        | Statement::ThrowStatement(_)
        | Statement::BreakStatement(_)
        | Statement::ContinueStatement(_) => true,
        Statement::BlockStatement(block) => ends_with_terminator(&block.body),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code with the `NoFallthrough` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoFallthrough)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_fallthrough() {
        let diags = lint("switch(x) { case 1: foo(); case 2: bar(); break; }");
        assert_eq!(diags.len(), 1, "case without break should be flagged");
    }

    #[test]
    fn test_allows_break() {
        let diags = lint("switch(x) { case 1: foo(); break; case 2: bar(); break; }");
        assert!(diags.is_empty(), "cases with break should not be flagged");
    }

    #[test]
    fn test_allows_return() {
        let diags = lint("function f(x) { switch(x) { case 1: return 1; case 2: return 2; } }");
        assert!(diags.is_empty(), "cases with return should not be flagged");
    }

    #[test]
    fn test_allows_throw() {
        let diags = lint("switch(x) { case 1: throw new Error(); case 2: break; }");
        assert!(diags.is_empty(), "cases with throw should not be flagged");
    }

    #[test]
    fn test_allows_empty_case_grouping() {
        let diags = lint("switch(x) { case 1: case 2: foo(); break; }");
        assert!(
            diags.is_empty(),
            "empty case grouping should not be flagged"
        );
    }

    #[test]
    fn test_allows_last_case_without_break() {
        let diags = lint("switch(x) { case 1: break; default: foo(); }");
        assert!(
            diags.is_empty(),
            "last case without break should not be flagged"
        );
    }
}
