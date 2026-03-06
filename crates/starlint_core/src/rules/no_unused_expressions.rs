//! Rule: `no-unused-expressions`
//!
//! Disallow expressions that have no side effects and whose value is
//! not used. Such expressions are likely mistakes or dead code.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags expression statements with no side effects.
#[derive(Debug)]
pub struct NoUnusedExpressions;

impl NativeRule for NoUnusedExpressions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-expressions".to_owned(),
            description: "Disallow unused expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExpressionStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ExpressionStatement(stmt) = kind else {
            return;
        };

        // Allow directive-like string literals ("use strict", etc.)
        if matches!(&stmt.expression, Expression::StringLiteral(_)) {
            return;
        }

        if is_unused_expression(&stmt.expression) {
            let span = Span::new(stmt.span.start, stmt.span.end);
            let fix = FixBuilder::new("Remove unused expression", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "no-unused-expressions".to_owned(),
                message: "Expected an assignment or function call and instead saw an expression"
                    .to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is "unused" — has no side effects and its
/// value is discarded.
fn is_unused_expression(expr: &Expression<'_>) -> bool {
    match expr {
        // Literals, identifiers, binary/logical expressions, and template
        // literals are always unused as expression statements
        Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::Identifier(_)
        | Expression::BinaryExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::TemplateLiteral(_) => true,

        // Unary expressions that are not void/delete/typeof are unused
        // (void, delete, typeof have side effects or are intentional)
        Expression::UnaryExpression(unary) => !matches!(
            unary.operator,
            oxc_ast::ast::UnaryOperator::Void
                | oxc_ast::ast::UnaryOperator::Delete
                | oxc_ast::ast::UnaryOperator::Typeof
        ),

        // Ternary/conditional — unused if both branches are unused
        Expression::ConditionalExpression(cond) => {
            is_unused_expression(&cond.consequent) && is_unused_expression(&cond.alternate)
        }

        // Sequence expressions — check the last expression
        Expression::SequenceExpression(seq) => seq
            .expressions
            .last()
            .is_some_and(|last| is_unused_expression(last)),

        // These have side effects and are not unused:
        // AssignmentExpression, CallExpression, NewExpression,
        // UpdateExpression (++/--), YieldExpression, AwaitExpression,
        // TaggedTemplateExpression (calls a function)
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnusedExpressions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_standalone_literal() {
        let diags = lint("42;");
        assert_eq!(diags.len(), 1, "standalone number should be flagged");
    }

    #[test]
    fn test_flags_standalone_identifier() {
        let diags = lint("foo;");
        assert_eq!(diags.len(), 1, "standalone identifier should be flagged");
    }

    #[test]
    fn test_flags_binary_expression() {
        let diags = lint("a + b;");
        assert_eq!(diags.len(), 1, "binary expression should be flagged");
    }

    #[test]
    fn test_allows_function_call() {
        let diags = lint("foo();");
        assert!(diags.is_empty(), "function call should not be flagged");
    }

    #[test]
    fn test_allows_assignment() {
        let diags = lint("x = 1;");
        assert!(diags.is_empty(), "assignment should not be flagged");
    }

    #[test]
    fn test_allows_increment() {
        let diags = lint("i++;");
        assert!(diags.is_empty(), "increment should not be flagged");
    }

    #[test]
    fn test_allows_use_strict() {
        let diags = lint("\"use strict\";");
        assert!(diags.is_empty(), "directive strings should not be flagged");
    }

    #[test]
    fn test_allows_delete() {
        let diags = lint("delete obj.prop;");
        assert!(diags.is_empty(), "delete should not be flagged");
    }

    #[test]
    fn test_allows_await() {
        let diags = lint("async function foo() { await bar; }");
        assert!(diags.is_empty(), "await should not be flagged");
    }
}
