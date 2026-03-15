//! Rule: `getter-return`
//!
//! Enforce `return` statements in getters. A getter without a `return`
//! statement implicitly returns `undefined`, which is almost always a bug.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{MethodDefinitionKind, PropertyKind};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags getters that don't contain a return statement.
#[derive(Debug)]
pub struct GetterReturn;

impl LintRule for GetterReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "getter-return".to_owned(),
            description: "Enforce `return` statements in getters".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition, AstNodeType::ObjectProperty])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::MethodDefinition(method) if method.kind == MethodDefinitionKind::Get => {
                // method.value is a NodeId pointing to a Function node
                let Some(AstNode::Function(func)) = ctx.node(method.value) else {
                    return;
                };
                let Some(body_id) = func.body else {
                    return;
                };
                let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                    return;
                };
                if !statements_contain_return(ctx, &body.statements) {
                    let span = Span::new(method.span.start, method.span.end);
                    ctx.report(Diagnostic {
                        rule_name: "getter-return".to_owned(),
                        message: "Expected a return value in getter".to_owned(),
                        span,
                        severity: Severity::Error,
                        help: Some("Add a `return` statement to this getter".to_owned()),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::ObjectProperty(prop) if prop.kind == PropertyKind::Get => {
                let Some(AstNode::Function(func)) = ctx.node(prop.value) else {
                    return;
                };
                let Some(body_id) = func.body else {
                    return;
                };
                let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                    return;
                };
                if !statements_contain_return(ctx, &body.statements) {
                    let span = Span::new(prop.span.start, prop.span.end);
                    ctx.report(Diagnostic {
                        rule_name: "getter-return".to_owned(),
                        message: "Expected a return value in getter".to_owned(),
                        span,
                        severity: Severity::Error,
                        help: Some("Add a `return` statement to this getter".to_owned()),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Check if any statement in the list contains a return statement with a value.
fn statements_contain_return(ctx: &LintContext<'_>, stmt_ids: &[NodeId]) -> bool {
    stmt_ids.iter().any(|&id| {
        ctx.node(id)
            .is_some_and(|s| statement_contains_return(ctx, s))
    })
}

/// Recursively check a single statement for a return with a value.
fn statement_contains_return(ctx: &LintContext<'_>, stmt: &AstNode) -> bool {
    match stmt {
        AstNode::ReturnStatement(ret) => ret.argument.is_some(),
        AstNode::BlockStatement(block) => statements_contain_return(ctx, &block.body),
        AstNode::IfStatement(if_stmt) => {
            let cons_has = ctx
                .node(if_stmt.consequent)
                .is_some_and(|n| statement_contains_return(ctx, n));
            let alt_has = if_stmt
                .alternate
                .and_then(|id| ctx.node(id))
                .is_some_and(|n| statement_contains_return(ctx, n));
            cons_has || alt_has
        }
        AstNode::SwitchStatement(switch) => switch.cases.iter().any(|&case_id| {
            ctx.node(case_id).is_some_and(|case_node| {
                if let AstNode::SwitchCase(case) = case_node {
                    case.consequent.iter().any(|&s_id| {
                        ctx.node(s_id)
                            .is_some_and(|s| statement_contains_return(ctx, s))
                    })
                } else {
                    false
                }
            })
        }),
        AstNode::TryStatement(try_stmt) => {
            let block_has = ctx.node(try_stmt.block).is_some_and(|n| {
                if let AstNode::BlockStatement(block) = n {
                    statements_contain_return(ctx, &block.body)
                } else {
                    false
                }
            });
            let handler_has = try_stmt
                .handler
                .and_then(|id| ctx.node(id))
                .is_some_and(|n| {
                    if let AstNode::CatchClause(catch) = n {
                        ctx.node(catch.body).is_some_and(|body_node| {
                            if let AstNode::BlockStatement(block) = body_node {
                                statements_contain_return(ctx, &block.body)
                            } else {
                                false
                            }
                        })
                    } else {
                        false
                    }
                });
            block_has || handler_has
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(GetterReturn);

    #[test]
    fn test_flags_getter_without_return() {
        let diags = lint("class Foo { get bar() { console.log('hi'); } }");
        assert_eq!(diags.len(), 1, "getter without return should be flagged");
    }

    #[test]
    fn test_allows_getter_with_return() {
        let diags = lint("class Foo { get bar() { return 1; } }");
        assert!(diags.is_empty(), "getter with return should not be flagged");
    }

    #[test]
    fn test_flags_object_getter_without_return() {
        let diags = lint("var obj = { get foo() { console.log('hi'); } };");
        assert_eq!(
            diags.len(),
            1,
            "object getter without return should be flagged"
        );
    }

    #[test]
    fn test_allows_object_getter_with_return() {
        let diags = lint("var obj = { get foo() { return 1; } };");
        assert!(
            diags.is_empty(),
            "object getter with return should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_in_if() {
        let diags = lint("class Foo { get bar() { if (true) { return 1; } } }");
        assert!(
            diags.is_empty(),
            "getter with return in if should not be flagged"
        );
    }

    #[test]
    fn test_allows_setter_without_return() {
        let diags = lint("class Foo { set bar(v) { this.x = v; } }");
        assert!(
            diags.is_empty(),
            "setter without return should not be flagged"
        );
    }

    #[test]
    fn test_allows_getter_with_return_in_try() {
        let diags = lint("class Foo { get bar() { try { return 1; } catch(e) {} } }");
        assert!(
            diags.is_empty(),
            "getter with return in try block should not be flagged"
        );
    }

    #[test]
    fn test_allows_getter_with_return_in_if_else() {
        let diags = lint("class Foo { get bar() { if(x) { return 1; } else { return 2; } } }");
        assert!(
            diags.is_empty(),
            "getter with return in both if and else branches should not be flagged"
        );
    }

    #[test]
    fn test_allows_object_getter_with_return_in_if_else() {
        let diags = lint("var obj = { get foo() { if(x) { return 1; } else { return 2; } } };");
        assert!(
            diags.is_empty(),
            "object getter with return in both if and else branches should not be flagged"
        );
    }

    #[test]
    fn test_allows_object_getter_with_return_in_if_only() {
        // The rule checks for the *existence* of a return, not exhaustive path coverage.
        // A return in one branch is sufficient to pass the check.
        let diags = lint("var obj = { get foo() { if(x) { return 1; } } };");
        assert!(
            diags.is_empty(),
            "object getter with return in if branch should not be flagged (existence check)"
        );
    }
}
