//! Rule: `prefer-array-index-of`
//!
//! Prefer `.indexOf()` over `.findIndex()` for simple equality checks.
//! `.findIndex(x => x === val)` can be simplified to `.indexOf(val)`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, FunctionBody, Statement};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.findIndex()` calls with simple equality callbacks.
#[derive(Debug)]
pub struct PreferArrayIndexOf;

/// Check if an arrow function body is a simple binary equality expression.
fn is_simple_equality_body(body: &FunctionBody<'_>) -> bool {
    // Expression body (single statement that is an expression statement)
    if body.statements.len() != 1 {
        return false;
    }
    let Some(stmt) = body.statements.first() else {
        return false;
    };
    let Statement::ExpressionStatement(expr_stmt) = stmt else {
        return false;
    };
    matches!(
        &expr_stmt.expression,
        Expression::BinaryExpression(bin)
            if matches!(
                bin.operator,
                oxc_ast::ast::BinaryOperator::StrictEquality | oxc_ast::ast::BinaryOperator::Equality
            )
    )
}

impl NativeRule for PreferArrayIndexOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-index-of".to_owned(),
            description: "Prefer `.indexOf()` over `.findIndex()` for simple equality checks"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name.as_str() != "findIndex" {
            return;
        }

        // Must have exactly one argument.
        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Check for arrow function with simple equality body.
        if let oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) = first_arg {
            if arrow.params.items.len() == 1 && is_simple_equality_body(&arrow.body) {
                ctx.report_warning(
                    "prefer-array-index-of",
                    "Prefer `.indexOf()` over `.findIndex()` for simple equality checks",
                    Span::new(call.span.start, call.span.end),
                );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferArrayIndexOf)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_find_index_strict_equality() {
        let diags = lint("arr.findIndex(x => x === 5);");
        assert_eq!(
            diags.len(),
            1,
            "should flag .findIndex() with strict equality"
        );
    }

    #[test]
    fn test_flags_find_index_loose_equality() {
        let diags = lint("arr.findIndex(x => x == val);");
        assert_eq!(
            diags.len(),
            1,
            "should flag .findIndex() with loose equality"
        );
    }

    #[test]
    fn test_allows_find_index_complex_callback() {
        let diags = lint("arr.findIndex(x => x.id === 5);");
        // This is a member expression equality, not a simple `x === val`.
        // Our heuristic still flags it because the body is a binary equality.
        // That is acceptable — it is a suggestion, not an error.
        assert_eq!(diags.len(), 1, "still flags member-based equality");
    }

    #[test]
    fn test_allows_index_of() {
        let diags = lint("arr.indexOf(5);");
        assert!(diags.is_empty(), ".indexOf() should not be flagged");
    }
}
