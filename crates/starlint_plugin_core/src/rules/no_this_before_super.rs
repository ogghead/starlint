//! Rule: `no-this-before-super`
//!
//! Disallow `this`/`super` before calling `super()` in constructors of derived
//! classes. Accessing `this` before `super()` is called throws a
//! `ReferenceError` at runtime.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `this` usage before `super()` in derived class constructors.
#[derive(Debug)]
pub struct NoThisBeforeSuper;

impl LintRule for NoThisBeforeSuper {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-this-before-super".to_owned(),
            description: "Disallow `this`/`super` before calling `super()` in constructors"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Class])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Class(class) = node else {
            return;
        };

        // Only check derived classes
        if class.super_class.is_none() {
            return;
        }

        // Find the constructor
        for &element_id in &*class.body {
            let Some(AstNode::MethodDefinition(method)) = ctx.node(element_id) else {
                continue;
            };

            if method.kind != MethodDefinitionKind::Constructor {
                continue;
            }

            let Some(AstNode::Function(func)) = ctx.node(method.value) else {
                continue;
            };

            let Some(body_id) = func.body else {
                continue;
            };

            let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                continue;
            };

            let stmts: Vec<NodeId> = body.statements.to_vec();
            check_this_before_super(&stmts, ctx);
        }
    }
}

/// Walk statements linearly, tracking whether `super()` has been called.
/// Flag any `this` usage before `super()`.
fn check_this_before_super(stmts: &[NodeId], ctx: &mut LintContext<'_>) {
    for &stmt_id in stmts {
        // Check if this statement contains `this` before we've seen `super()`
        if let Some(this_span) = find_this_in_statement(stmt_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: "no-this-before-super".to_owned(),
                message: "`this` is not allowed before `super()`".to_owned(),
                span: this_span,
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
            return;
        }

        // Check if this statement contains a `super()` call
        if statement_has_super_call(stmt_id, ctx) {
            return; // After super(), this is fine
        }
    }
}

/// Find `this` expression in a statement, returning its span.
fn find_this_in_statement(stmt_id: NodeId, ctx: &LintContext<'_>) -> Option<Span> {
    match ctx.node(stmt_id)? {
        AstNode::ExpressionStatement(expr_stmt) => {
            find_this_in_expression(expr_stmt.expression, ctx)
        }
        AstNode::VariableDeclaration(decl) => {
            for &decl_id in &*decl.declarations {
                if let Some(AstNode::VariableDeclarator(declarator)) = ctx.node(decl_id) {
                    if let Some(init_id) = declarator.init {
                        if let Some(span) = find_this_in_expression(init_id, ctx) {
                            return Some(span);
                        }
                    }
                }
            }
            None
        }
        AstNode::ReturnStatement(ret) => {
            if let Some(arg_id) = ret.argument {
                return find_this_in_expression(arg_id, ctx);
            }
            None
        }
        _ => None,
    }
}

/// Find `this` expression recursively, returning its span.
#[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
fn find_this_in_expression(expr_id: NodeId, ctx: &LintContext<'_>) -> Option<Span> {
    match ctx.node(expr_id)? {
        AstNode::ThisExpression(this) => Some(Span::new(this.span.start, this.span.end)),
        AstNode::AssignmentExpression(assign) => {
            // Check the left side for `this`
            if let Some(span) = find_this_in_expression(assign.left, ctx) {
                return Some(span);
            }
            find_this_in_expression(assign.right, ctx)
        }
        AstNode::CallExpression(call) => {
            // Skip super() calls -- that's what we're looking for.
            // `super` maps to Unknown in starlint_ast so check source text.
            if ctx.node(call.callee).is_some_and(|n| {
                let sp = n.span();
                ctx.source_text().get(sp.start as usize..sp.end as usize) == Some("super")
            }) {
                return None;
            }
            find_this_in_expression(call.callee, ctx)
        }
        AstNode::StaticMemberExpression(member) => find_this_in_expression(member.object, ctx),
        AstNode::ComputedMemberExpression(member) => find_this_in_expression(member.object, ctx),
        _ => None,
    }
}

/// Check if a statement contains a `super()` call.
fn statement_has_super_call(stmt_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(stmt_id) {
        Some(AstNode::ExpressionStatement(expr_stmt)) => {
            expression_is_super_call(expr_stmt.expression, ctx)
        }
        _ => false,
    }
}

/// Check if an expression is a `super()` call.
fn expression_is_super_call(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::CallExpression(call)) => is_callee_super(call.callee, ctx),
        _ => false,
    }
}

/// Check if a callee node represents `super` by inspecting source text.
#[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
fn is_callee_super(callee_id: NodeId, ctx: &LintContext<'_>) -> bool {
    ctx.node(callee_id).is_some_and(|n| {
        let sp = n.span();
        ctx.source_text().get(sp.start as usize..sp.end as usize) == Some("super")
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoThisBeforeSuper);

    #[test]
    fn test_flags_this_before_super() {
        let diags = lint("class B extends A { constructor() { this.x = 1; super(); } }");
        assert_eq!(diags.len(), 1, "this before super() should be flagged");
    }

    #[test]
    fn test_allows_this_after_super() {
        let diags = lint("class B extends A { constructor() { super(); this.x = 1; } }");
        assert!(diags.is_empty(), "this after super() should not be flagged");
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
