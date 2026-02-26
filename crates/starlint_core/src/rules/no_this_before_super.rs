//! Rule: `no-this-before-super`
//!
//! Disallow `this`/`super` before calling `super()` in constructors of derived
//! classes. Accessing `this` before `super()` is called throws a
//! `ReferenceError` at runtime.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentTarget, ClassElement, Expression, MethodDefinitionKind, Statement};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `this` usage before `super()` in derived class constructors.
#[derive(Debug)]
pub struct NoThisBeforeSuper;

impl NativeRule for NoThisBeforeSuper {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-this-before-super".to_owned(),
            description: "Disallow `this`/`super` before calling `super()` in constructors"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        // Only check derived classes
        if class.super_class.is_none() {
            return;
        }

        // Find the constructor
        for element in &class.body.body {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };

            if method.kind != MethodDefinitionKind::Constructor {
                continue;
            }

            let Some(body) = &method.value.body else {
                continue;
            };

            check_this_before_super(&body.statements, ctx);
        }
    }
}

/// Walk statements linearly, tracking whether `super()` has been called.
/// Flag any `this` usage before `super()`.
fn check_this_before_super(stmts: &[Statement<'_>], ctx: &mut NativeLintContext<'_>) {
    for stmt in stmts {
        // Check if this statement contains `this` before we've seen `super()`
        if let Some(this_span) = find_this_in_statement(stmt) {
            ctx.report_error(
                "no-this-before-super",
                "`this` is not allowed before `super()`",
                this_span,
            );
            return;
        }

        // Check if this statement contains a `super()` call
        if statement_has_super_call(stmt) {
            return; // After super(), this is fine
        }
    }
}

/// Find `this` expression in a statement, returning its span.
fn find_this_in_statement(stmt: &Statement<'_>) -> Option<Span> {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => find_this_in_expression(&expr_stmt.expression),
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    if let Some(span) = find_this_in_expression(init) {
                        return Some(span);
                    }
                }
            }
            None
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                return find_this_in_expression(arg);
            }
            None
        }
        _ => None,
    }
}

/// Find `this` expression recursively, returning its span.
fn find_this_in_expression(expr: &Expression<'_>) -> Option<Span> {
    match expr {
        Expression::ThisExpression(this) => Some(Span::new(this.span.start, this.span.end)),
        Expression::AssignmentExpression(assign) => {
            // Check the assignment target (left side) for `this`
            if let Some(span) = find_this_in_target(&assign.left) {
                return Some(span);
            }
            find_this_in_expression(&assign.right)
        }
        Expression::CallExpression(call) => {
            // Skip super() calls — that's what we're looking for
            if matches!(&call.callee, Expression::Super(_)) {
                return None;
            }
            find_this_in_expression(&call.callee)
        }
        Expression::StaticMemberExpression(member) => {
            find_this_in_expression(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            find_this_in_expression(&member.object)
        }
        _ => None,
    }
}

/// Find `this` in an assignment target (left side of assignment).
fn find_this_in_target(target: &AssignmentTarget<'_>) -> Option<Span> {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => {
            find_this_in_expression(&member.object)
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            find_this_in_expression(&member.object)
        }
        _ => None,
    }
}

/// Check if a statement contains a `super()` call.
fn statement_has_super_call(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            expression_is_super_call(&expr_stmt.expression)
        }
        _ => false,
    }
}

/// Check if an expression is a `super()` call.
fn expression_is_super_call(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::CallExpression(call) => matches!(&call.callee, Expression::Super(_)),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoThisBeforeSuper)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_this_before_super() {
        let diags = lint("class B extends A { constructor() { this.x = 1; super(); } }");
        assert_eq!(
            diags.len(),
            1,
            "this before super() should be flagged"
        );
    }

    #[test]
    fn test_allows_this_after_super() {
        let diags = lint("class B extends A { constructor() { super(); this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "this after super() should not be flagged"
        );
    }

    #[test]
    fn test_allows_base_class() {
        let diags = lint("class A { constructor() { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "base class constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_this() {
        let diags = lint("class B extends A { constructor() { super(); } }");
        assert!(
            diags.is_empty(),
            "constructor without this should not be flagged"
        );
    }

    #[test]
    fn test_flags_member_access_before_super() {
        let diags = lint("class B extends A { constructor() { this.foo(); super(); } }");
        assert_eq!(
            diags.len(),
            1,
            "this.foo() before super() should be flagged"
        );
    }
}
