//! Rule: `no-setter-return`
//!
//! Disallow returning a value from a setter. Setters cannot return a value;
//! any `return <expr>` inside a setter is ignored and indicates a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{MethodDefinitionKind, PropertyKind};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `return <value>` statements inside setter functions.
#[derive(Debug)]
pub struct NoSetterReturn;

impl LintRule for NoSetterReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-setter-return".to_owned(),
            description: "Disallow returning a value from a setter".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition, AstNodeType::ObjectProperty])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::MethodDefinition(method) if method.kind == MethodDefinitionKind::Set => {
                // method.value is a NodeId pointing to a Function node
                if let Some(AstNode::Function(func)) = ctx.node(method.value) {
                    if let Some(body_id) = func.body {
                        if let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) {
                            let stmts = body.statements.clone();
                            check_statements_for_value_return(&stmts, ctx);
                        }
                    }
                }
            }
            AstNode::ObjectProperty(prop) if prop.kind == PropertyKind::Set => {
                if let Some(AstNode::Function(func)) = ctx.node(prop.value) {
                    if let Some(body_id) = func.body {
                        if let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) {
                            let stmts = body.statements.clone();
                            check_statements_for_value_return(&stmts, ctx);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Walk statements looking for return statements that have a value.
fn check_statements_for_value_return(stmts: &[NodeId], ctx: &mut LintContext<'_>) {
    for stmt_id in stmts {
        check_statement_for_value_return(*stmt_id, ctx);
    }
}

/// Check a single statement for `return <value>`.
fn check_statement_for_value_return(stmt_id: NodeId, ctx: &mut LintContext<'_>) {
    let Some(stmt) = ctx.node(stmt_id) else {
        return;
    };
    match stmt {
        AstNode::ReturnStatement(ret) => {
            if ret.argument.is_some() {
                let ret_span = Span::new(ret.span.start, ret.span.end);
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with bare `return;`".to_owned(),
                    edits: vec![Edit {
                        span: ret_span,
                        replacement: "return;".to_owned(),
                    }],
                    is_snippet: false,
                });
                ctx.report(Diagnostic {
                    rule_name: "no-setter-return".to_owned(),
                    message: "Setter cannot return a value".to_owned(),
                    span: ret_span,
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
        AstNode::BlockStatement(block) => {
            let body = block.body.clone();
            check_statements_for_value_return(&body, ctx);
        }
        AstNode::IfStatement(if_stmt) => {
            let consequent = if_stmt.consequent;
            let alternate = if_stmt.alternate;
            check_statement_for_value_return(consequent, ctx);
            if let Some(alt) = alternate {
                check_statement_for_value_return(alt, ctx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoSetterReturn);

    #[test]
    fn test_flags_setter_return_value() {
        let diags = lint("class Foo { set bar(v) { return v; } }");
        assert_eq!(diags.len(), 1, "setter returning value should be flagged");
    }

    #[test]
    fn test_allows_setter_bare_return() {
        let diags = lint("class Foo { set bar(v) { this.x = v; return; } }");
        assert!(
            diags.is_empty(),
            "bare return in setter should not be flagged"
        );
    }

    #[test]
    fn test_allows_getter_return() {
        let diags = lint("class Foo { get bar() { return 1; } }");
        assert!(
            diags.is_empty(),
            "getter returning value should not be flagged"
        );
    }

    #[test]
    fn test_flags_object_setter_return() {
        let diags = lint("var obj = { set foo(v) { return v; } };");
        assert_eq!(
            diags.len(),
            1,
            "object setter returning value should be flagged"
        );
    }

    #[test]
    fn test_allows_method_return() {
        let diags = lint("class Foo { bar() { return 1; } }");
        assert!(
            diags.is_empty(),
            "normal method return should not be flagged"
        );
    }
}
