//! Rule: `no-ex-assign`
//!
//! Disallow reassigning exceptions in `catch` clauses. Overwriting the
//! caught exception destroys the original error information and is almost
//! always a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags assignments to the catch clause parameter.
#[derive(Debug)]
pub struct NoExAssign;

impl LintRule for NoExAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-ex-assign".to_owned(),
            description: "Disallow reassigning exceptions in catch clauses".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CatchClause])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CatchClause(catch) = node else {
            return;
        };

        // Get the name of the catch parameter
        let Some(param_id) = catch.param else {
            return;
        };

        let Some(AstNode::BindingIdentifier(ident)) = ctx.node(param_id) else {
            return;
        };

        let param_name = ident.name.clone();

        // Scan the catch body for assignments to the parameter name
        scan_body_for_assignment(catch.body, &param_name, ctx);
    }
}

/// Scan a block body for assignments to a given identifier.
fn scan_body_for_assignment(body_id: NodeId, name: &str, ctx: &mut LintContext<'_>) {
    let Some(AstNode::BlockStatement(block)) = ctx.node(body_id) else {
        return;
    };
    let body = block.body.clone();
    for stmt_id in &body {
        scan_statement_for_assignment(*stmt_id, name, ctx);
    }
}

/// Check a single statement for assignments to the named identifier.
fn scan_statement_for_assignment(stmt_id: NodeId, name: &str, ctx: &mut LintContext<'_>) {
    let Some(stmt) = ctx.node(stmt_id) else {
        return;
    };
    match stmt {
        AstNode::ExpressionStatement(expr_stmt) => {
            let expr_id = expr_stmt.expression;
            if let Some(AstNode::AssignmentExpression(assign)) = ctx.node(expr_id) {
                let left_id = assign.left;
                let assign_span = assign.span;
                if let Some(AstNode::IdentifierReference(target_ident)) = ctx.node(left_id) {
                    if target_ident.name.as_str() == name {
                        ctx.report(Diagnostic {
                            rule_name: "no-ex-assign".to_owned(),
                            message: format!("Do not assign to the exception parameter `{name}`"),
                            span: Span::new(assign_span.start, assign_span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
        }
        AstNode::BlockStatement(block) => {
            let body = block.body.clone();
            for inner_id in &body {
                scan_statement_for_assignment(*inner_id, name, ctx);
            }
        }
        AstNode::IfStatement(if_stmt) => {
            let consequent = if_stmt.consequent;
            let alternate = if_stmt.alternate;
            scan_statement_for_assignment(consequent, name, ctx);
            if let Some(alt_id) = alternate {
                scan_statement_for_assignment(alt_id, name, ctx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoExAssign);

    #[test]
    fn test_flags_catch_param_reassign() {
        let diags = lint("try {} catch (e) { e = 10; }");
        assert_eq!(diags.len(), 1, "reassigning catch param should be flagged");
    }

    #[test]
    fn test_allows_catch_param_usage() {
        let diags = lint("try {} catch (e) { console.log(e); }");
        assert!(diags.is_empty(), "using catch param should not be flagged");
    }

    #[test]
    fn test_allows_different_variable_assignment() {
        let diags = lint("try {} catch (e) { let x = 10; }");
        assert!(
            diags.is_empty(),
            "assigning to different variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_catch_param() {
        let diags = lint("try {} catch { let x = 1; }");
        assert!(
            diags.is_empty(),
            "catch without param should not be flagged"
        );
    }

    #[test]
    fn test_flags_nested_reassign() {
        let diags = lint("try {} catch (e) { if (true) { e = 10; } }");
        assert_eq!(
            diags.len(),
            1,
            "nested reassign of catch param should be flagged"
        );
    }
}
