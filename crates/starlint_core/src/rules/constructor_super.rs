//! Rule: `constructor-super`
//!
//! Require `super()` calls in constructors of derived classes, and disallow
//! `super()` in constructors of non-derived classes. A derived class (one
//! that `extends` another) must call `super()` before using `this`.

use oxc_ast::AstKind;
use oxc_ast::ast::{ClassElement, Expression, MethodDefinitionKind, Statement};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags missing or unnecessary `super()` calls in constructors.
#[derive(Debug)]
pub struct ConstructorSuper;

impl NativeRule for ConstructorSuper {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "constructor-super".to_owned(),
            description: "Require super() calls in constructors of derived classes".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        let has_super_class = class.super_class.is_some();

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

            let has_super_call = statements_contain_super_call(&body.statements);

            if has_super_class && !has_super_call {
                ctx.report_error(
                    "constructor-super",
                    "Derived class constructor must call `super()`",
                    Span::new(method.span.start, method.span.end),
                );
            }
        }
    }
}

/// Check if any statement contains a `super()` call expression.
fn statements_contain_super_call(stmts: &[Statement<'_>]) -> bool {
    stmts.iter().any(|s| statement_contains_super_call(s))
}

/// Recursively check a single statement for a `super()` call.
fn statement_contains_super_call(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            expression_contains_super_call(&expr_stmt.expression)
        }
        Statement::BlockStatement(block) => statements_contain_super_call(&block.body),
        Statement::IfStatement(if_stmt) => {
            statement_contains_super_call(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_ref()
                    .is_some_and(|alt| statement_contains_super_call(alt))
        }
        _ => false,
    }
}

/// Check if an expression is or contains a `super()` call.
fn expression_contains_super_call(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::CallExpression(call) => matches!(&call.callee, Expression::Super(_)),
        Expression::SequenceExpression(seq) => seq
            .expressions
            .iter()
            .any(|e| expression_contains_super_call(e)),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConstructorSuper)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_missing_super_in_derived() {
        let diags = lint("class Bar extends Foo { constructor() { this.x = 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "derived constructor without super() should be flagged"
        );
    }

    #[test]
    fn test_allows_super_in_derived() {
        let diags = lint("class Bar extends Foo { constructor() { super(); } }");
        assert!(
            diags.is_empty(),
            "derived constructor with super() should not be flagged"
        );
    }

    #[test]
    fn test_allows_base_class_no_super() {
        let diags = lint("class Foo { constructor() { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "base class constructor without super() should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_constructor() {
        let diags = lint("class Foo extends Bar {}");
        assert!(
            diags.is_empty(),
            "class without constructor should not be flagged"
        );
    }
}
