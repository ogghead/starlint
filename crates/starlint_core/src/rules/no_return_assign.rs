//! Rule: `no-return-assign`
//!
//! Disallow assignment operators in `return` statements. Using assignment
//! in a return is often a mistake (intended `===` comparison).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags assignment expressions in `return` statements.
#[derive(Debug)]
pub struct NoReturnAssign;

impl NativeRule for NoReturnAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-return-assign".to_owned(),
            description: "Disallow assignment operators in `return` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ReturnStatement(ret) = kind else {
            return;
        };

        let Some(arg) = &ret.argument else {
            return;
        };

        if contains_assignment(arg) {
            ctx.report_error(
                "no-return-assign",
                "Assignment in return statement — use a separate statement or `===` for comparison",
                Span::new(ret.span.start, ret.span.end),
            );
        }
    }
}

/// Check if an expression contains an assignment operator.
fn contains_assignment(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::AssignmentExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => contains_assignment(&paren.expression),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoReturnAssign)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_assignment() {
        let diags = lint("function f() { return x = 1; }");
        assert_eq!(diags.len(), 1, "return with assignment should be flagged");
    }

    #[test]
    fn test_flags_parenthesized_assignment() {
        let diags = lint("function f() { return (x = 1); }");
        assert_eq!(
            diags.len(),
            1,
            "return with parenthesized assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_return_value() {
        let diags = lint("function f() { return x + 1; }");
        assert!(
            diags.is_empty(),
            "return with expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_comparison() {
        let diags = lint("function f() { return x === 1; }");
        assert!(
            diags.is_empty(),
            "return with comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("function f() { return; }");
        assert!(diags.is_empty(), "empty return should not be flagged");
    }
}
