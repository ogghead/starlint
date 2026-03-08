//! Rule: `no-accessor-recursion` (unicorn)
//!
//! Disallow recursive getters and setters. A getter that accesses its own
//! property or a setter that assigns to its own property causes infinite
//! recursion.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags getters/setters that recursively access/assign their own property.
#[derive(Debug)]
pub struct NoAccessorRecursion;

impl LintRule for NoAccessorRecursion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-accessor-recursion".to_owned(),
            description: "Disallow recursive getters and setters".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        // Get property name from the key
        let prop_name = ctx.node(method.key).and_then(|n| {
            if let AstNode::IdentifierReference(id) = n {
                Some(id.name.clone())
            } else if let AstNode::BindingIdentifier(id) = n {
                Some(id.name.clone())
            } else {
                None
            }
        });

        let Some(prop_name) = prop_name else {
            return;
        };

        let method_span = method.span;

        match method.kind {
            MethodDefinitionKind::Get => {
                // Check if the getter body accesses `this.propName`
                let Some(AstNode::Function(func)) = ctx.node(method.value) else {
                    return;
                };
                let Some(body_id) = func.body else {
                    return;
                };
                let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                    return;
                };
                let stmt_ids: Vec<NodeId> = body.statements.to_vec();

                for stmt_id in stmt_ids {
                    if statement_accesses_this_property(ctx, stmt_id, &prop_name) {
                        ctx.report(Diagnostic {
                            rule_name: "no-accessor-recursion".to_owned(),
                            message: format!(
                                "Getter for '{prop_name}' recursively accesses `this.{prop_name}`"
                            ),
                            span: Span::new(method_span.start, method_span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                        return;
                    }
                }
            }
            MethodDefinitionKind::Set => {
                // Check if the setter body assigns to `this.propName`
                let Some(AstNode::Function(func)) = ctx.node(method.value) else {
                    return;
                };
                let Some(body_id) = func.body else {
                    return;
                };
                let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                    return;
                };
                let stmt_ids: Vec<NodeId> = body.statements.to_vec();

                for stmt_id in stmt_ids {
                    if statement_assigns_this_property(ctx, stmt_id, &prop_name) {
                        ctx.report(Diagnostic {
                            rule_name: "no-accessor-recursion".to_owned(),
                            message: format!(
                                "Setter for '{prop_name}' recursively assigns to `this.{prop_name}`"
                            ),
                            span: Span::new(method_span.start, method_span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                        return;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if a statement reads `this.propName`.
fn statement_accesses_this_property(
    ctx: &LintContext<'_>,
    stmt_id: NodeId,
    prop_name: &str,
) -> bool {
    let Some(stmt) = ctx.node(stmt_id) else {
        return false;
    };
    match stmt {
        AstNode::ReturnStatement(ret) => ret
            .argument
            .and_then(|id| ctx.node(id))
            .is_some_and(|n| expression_accesses_this_property(ctx, n, prop_name)),
        AstNode::ExpressionStatement(expr_stmt) => ctx
            .node(expr_stmt.expression)
            .is_some_and(|n| expression_accesses_this_property(ctx, n, prop_name)),
        _ => false,
    }
}

/// Check if an expression reads `this.propName`.
fn expression_accesses_this_property(
    ctx: &LintContext<'_>,
    node: &AstNode,
    prop_name: &str,
) -> bool {
    if let AstNode::StaticMemberExpression(member) = node {
        let is_this = ctx
            .node(member.object)
            .is_some_and(|n| matches!(n, AstNode::ThisExpression(_)));
        is_this && member.property == prop_name
    } else {
        false
    }
}

/// Check if a statement assigns to `this.propName`.
fn statement_assigns_this_property(
    ctx: &LintContext<'_>,
    stmt_id: NodeId,
    prop_name: &str,
) -> bool {
    let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(stmt_id) else {
        return false;
    };
    let Some(AstNode::AssignmentExpression(assign)) = ctx.node(expr_stmt.expression) else {
        return false;
    };
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(assign.left) else {
        return false;
    };
    let is_this = ctx
        .node(member.object)
        .is_some_and(|n| matches!(n, AstNode::ThisExpression(_)));
    is_this && member.property == prop_name
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAccessorRecursion)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_recursive_getter() {
        let diags = lint("class Foo { get bar() { return this.bar; } }");
        assert_eq!(diags.len(), 1, "recursive getter should be flagged");
    }

    #[test]
    fn test_flags_recursive_setter() {
        let diags = lint("class Foo { set bar(val) { this.bar = val; } }");
        assert_eq!(diags.len(), 1, "recursive setter should be flagged");
    }

    #[test]
    fn test_allows_non_recursive_getter() {
        let diags = lint("class Foo { get bar() { return this._bar; } }");
        assert!(
            diags.is_empty(),
            "non-recursive getter should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_recursive_setter() {
        let diags = lint("class Foo { set bar(val) { this._bar = val; } }");
        assert!(
            diags.is_empty(),
            "non-recursive setter should not be flagged"
        );
    }
}
