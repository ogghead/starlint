//! Rule: `guard-for-in`
//!
//! Require `hasOwnProperty` checks in `for-in` loops. The `for-in` statement
//! iterates over all enumerable properties of an object, including inherited
//! ones. It is a common best practice to filter out inherited properties with
//! an `if` guard (e.g. `if (obj.hasOwnProperty(k))`) or a `continue` guard.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `for-in` loops that do not guard with an `if` statement or `continue`.
#[derive(Debug)]
pub struct GuardForIn;

impl NativeRule for GuardForIn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "guard-for-in".to_owned(),
            description: "Require `hasOwnProperty` check in `for-in` loops".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ForInStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ForInStatement(for_in) = kind else {
            return;
        };

        if is_guarded(&for_in.body) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "guard-for-in".to_owned(),
            message: "The body of a `for-in` should be wrapped in an `if` statement to filter unwanted properties from the prototype".to_owned(),
            span: Span::new(for_in.span.start, for_in.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check if the for-in body is guarded.
///
/// A body is considered guarded if:
/// - It is a block whose first statement is an `if` statement (guard pattern), OR
/// - It is a block whose only statement is a `continue` statement, OR
/// - It is directly an `if` statement (no block wrapper)
fn is_guarded(body: &Statement<'_>) -> bool {
    match body {
        Statement::BlockStatement(block) => {
            let stmts = &block.body;

            // Empty block — nothing to guard
            if stmts.is_empty() {
                return true;
            }

            // First statement is an if-statement — accepted as a guard
            if let Some(Statement::IfStatement(_)) = stmts.first() {
                return true;
            }

            // Single continue statement — accepted as a guard
            if stmts.len() == 1 {
                if let Some(Statement::ContinueStatement(_)) = stmts.first() {
                    return true;
                }
            }

            false
        }
        // If statement directly as body (no block), or empty statement — nothing to guard
        Statement::IfStatement(_) | Statement::EmptyStatement(_) => true,
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

    /// Helper to lint source code with the `GuardForIn` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(GuardForIn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_unguarded_for_in() {
        let diags = lint("for (var k in obj) { use(k); }");
        assert_eq!(diags.len(), 1, "for-in without guard should be flagged");
    }

    #[test]
    fn test_allows_if_guard() {
        let diags = lint("for (var k in obj) { if (obj.hasOwnProperty(k)) { use(k); } }");
        assert!(
            diags.is_empty(),
            "for-in with if guard should not be flagged"
        );
    }

    #[test]
    fn test_allows_if_continue_guard() {
        let diags = lint("for (var k in obj) { if (!obj.hasOwnProperty(k)) continue; use(k); }");
        assert!(
            diags.is_empty(),
            "for-in with if-continue guard should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_continue() {
        let diags = lint("for (var k in obj) { continue; }");
        assert!(
            diags.is_empty(),
            "for-in with only continue should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_body() {
        let diags = lint("for (var k in obj) { }");
        assert!(
            diags.is_empty(),
            "for-in with empty body should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_statements_no_guard() {
        let diags = lint("for (var k in obj) { foo(k); bar(k); }");
        assert_eq!(
            diags.len(),
            1,
            "for-in with multiple unguarded statements should be flagged"
        );
    }
}
