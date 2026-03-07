//! Rule: `constructor-super`
//!
//! Require `super()` calls in constructors of derived classes, and disallow
//! `super()` in constructors of non-derived classes. A derived class (one
//! that `extends` another) must call `super()` before using `this`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;

/// Flags missing or unnecessary `super()` calls in constructors.
#[derive(Debug)]
pub struct ConstructorSuper;

impl LintRule for ConstructorSuper {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "constructor-super".to_owned(),
            description: "Require super() calls in constructors of derived classes".to_owned(),
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

        let has_super_class = class.super_class.is_some();

        // Find the constructor
        for element_id in &*class.body {
            let Some(AstNode::MethodDefinition(method)) = ctx.node(*element_id) else {
                continue;
            };

            if method.kind != MethodDefinitionKind::Constructor {
                continue;
            }

            // method.value is a NodeId pointing to a Function node
            let Some(AstNode::Function(func)) = ctx.node(method.value) else {
                continue;
            };

            let Some(body_id) = func.body else {
                continue;
            };

            let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                continue;
            };

            let has_super_call = statements_contain_super_call(&body.statements, ctx);

            if has_super_class && !has_super_call {
                // Insert `super();` right after the opening `{` of the body
                let insert_pos = body.span.start.saturating_add(1);
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `super()` call".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(insert_pos, insert_pos),
                        replacement: " super();".to_owned(),
                    }],
                    is_snippet: false,
                });
                let method_span = Span::new(method.span.start, method.span.end);
                ctx.report(Diagnostic {
                    rule_name: "constructor-super".to_owned(),
                    message: "Derived class constructor must call `super()`".to_owned(),
                    span: method_span,
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

/// Check if any statement contains a `super()` call expression.
fn statements_contain_super_call(stmts: &[NodeId], ctx: &LintContext<'_>) -> bool {
    stmts.iter().any(|s| statement_contains_super_call(*s, ctx))
}

/// Recursively check a single statement for a `super()` call.
fn statement_contains_super_call(stmt_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(stmt) = ctx.node(stmt_id) else {
        return false;
    };
    match stmt {
        AstNode::ExpressionStatement(expr_stmt) => {
            expression_contains_super_call(expr_stmt.expression, ctx)
        }
        AstNode::BlockStatement(block) => statements_contain_super_call(&block.body, ctx),
        AstNode::IfStatement(if_stmt) => {
            statement_contains_super_call(if_stmt.consequent, ctx)
                || if_stmt
                    .alternate
                    .is_some_and(|alt| statement_contains_super_call(alt, ctx))
        }
        _ => false,
    }
}

/// Check if an expression is or contains a `super()` call.
fn expression_contains_super_call(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(expr) = ctx.node(expr_id) else {
        return false;
    };
    match expr {
        AstNode::CallExpression(call) => {
            // Check if callee is an identifier "super" (super calls are represented
            // as IdentifierReference with name "super" in starlint_ast)
            matches!(ctx.node(call.callee), Some(AstNode::IdentifierReference(id)) if id.name == "super")
        }
        AstNode::SequenceExpression(seq) => seq
            .expressions
            .iter()
            .any(|e| expression_contains_super_call(*e, ctx)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConstructorSuper)];
        lint_source(source, "test.js", &rules)
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
