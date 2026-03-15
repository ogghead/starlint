//! Rule: `no-param-reassign`
//!
//! Disallow reassignment of function parameters. Modifying parameters
//! can lead to confusing behavior and unexpected side effects.
//! This is a simplified version that flags direct assignment to parameter names.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags reassignment of function parameters (simplified).
#[derive(Debug)]
pub struct NoParamReassign;

impl LintRule for NoParamReassign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-param-reassign".to_owned(),
            description: "Disallow reassignment of function parameters".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Look for function declarations/expressions and check their body
        // for assignments to parameter names
        let (params, body_stmts) = match node {
            AstNode::Function(func) => {
                let Some(body_id) = func.body else {
                    return;
                };
                let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
                    return;
                };
                (&func.params, &*body.statements)
            }
            _ => return,
        };

        let param_names = collect_param_names(params, ctx);
        if param_names.is_empty() {
            return;
        }

        // Scan body for direct assignments to parameter names
        let source = ctx.source_text();
        let mut spans_to_report: Vec<(String, Span)> = Vec::new();

        for stmt_id in body_stmts {
            if let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(*stmt_id) {
                if let Some(AstNode::AssignmentExpression(assign)) = ctx.node(expr_stmt.expression)
                {
                    let (start, end) = match ctx.node(assign.left) {
                        Some(n) => {
                            let s = n.span();
                            (
                                usize::try_from(s.start).unwrap_or(0),
                                usize::try_from(s.end).unwrap_or(0),
                            )
                        }
                        None => (0, 0),
                    };
                    let target_text = source.get(start..end).unwrap_or("");

                    for name in &param_names {
                        if target_text == name.as_str() {
                            spans_to_report.push((
                                name.clone(),
                                Span::new(assign.span.start, assign.span.end),
                            ));
                        }
                    }
                }
            }
        }

        for (name, span) in spans_to_report {
            ctx.report(Diagnostic {
                rule_name: "no-param-reassign".to_owned(),
                message: format!("Assignment to function parameter `{name}`"),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Collect parameter names from formal parameters (Box<[`NodeId`]>).
fn collect_param_names(params: &[NodeId], ctx: &LintContext<'_>) -> Vec<String> {
    let mut names = Vec::new();
    for param_id in params {
        if let Some(AstNode::BindingIdentifier(id)) = ctx.node(*param_id) {
            names.push(id.name.clone());
        }
    }
    names
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoParamReassign);

    #[test]
    fn test_flags_param_reassign() {
        let diags = lint("function foo(x) { x = 10; }");
        assert_eq!(diags.len(), 1, "parameter reassignment should be flagged");
    }

    #[test]
    fn test_allows_local_variable() {
        let diags = lint("function foo(x) { var y = 10; }");
        assert!(diags.is_empty(), "local variable should not be flagged");
    }

    #[test]
    fn test_allows_no_reassign() {
        let diags = lint("function foo(x) { return x; }");
        assert!(
            diags.is_empty(),
            "using param without reassign should not be flagged"
        );
    }
}
