//! Rule: `no-constructor-return`
//!
//! Disallow returning a value from a constructor. Constructors should not use
//! `return <value>` — it interferes with the normal `new` operator behavior.
//! A bare `return;` is acceptable for early exit.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;

/// Flags `return <value>` statements inside class constructors.
#[derive(Debug)]
pub struct NoConstructorReturn;

impl LintRule for NoConstructorReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-constructor-return".to_owned(),
            description: "Disallow returning a value from a constructor".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }

        // method.value is NodeId -> Function -> body is Option<NodeId> -> FunctionBody
        let func_node = ctx.node(method.value);
        let Some(AstNode::Function(func)) = func_node else {
            return;
        };
        let Some(body_id) = func.body else {
            return;
        };
        let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
            return;
        };

        let stmt_ids: Vec<NodeId> = body.statements.to_vec();
        check_statements_for_value_return(&stmt_ids, ctx);
    }
}

/// Walk statements looking for return statements that have a value.
fn check_statements_for_value_return(stmt_ids: &[NodeId], ctx: &mut LintContext<'_>) {
    for stmt_id in stmt_ids {
        check_statement_for_value_return(*stmt_id, ctx);
    }
}

/// Check a single statement for `return <value>`.
fn check_statement_for_value_return(stmt_id: NodeId, ctx: &mut LintContext<'_>) {
    match ctx.node(stmt_id) {
        Some(AstNode::ReturnStatement(ret)) => {
            if ret.argument.is_some() {
                let ret_span = ret.span;
                ctx.report(Diagnostic {
                    rule_name: "no-constructor-return".to_owned(),
                    message: "Unexpected return statement in constructor".to_owned(),
                    span: Span::new(ret_span.start, ret_span.end),
                    severity: Severity::Error,
                    help: Some("Remove the return value or use a bare `return;`".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove the return value".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(ret_span.start, ret_span.end),
                            replacement: "return;".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
        Some(AstNode::BlockStatement(block)) => {
            let body_ids: Vec<NodeId> = block.body.to_vec();
            check_statements_for_value_return(&body_ids, ctx);
        }
        Some(AstNode::IfStatement(if_stmt)) => {
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConstructorReturn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_return_value_in_constructor() {
        let diags = lint("class Foo { constructor() { return {}; } }");
        assert_eq!(
            diags.len(),
            1,
            "return value in constructor should be flagged"
        );
    }

    #[test]
    fn test_allows_bare_return() {
        let diags = lint("class Foo { constructor() { return; } }");
        assert!(
            diags.is_empty(),
            "bare return in constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_return() {
        let diags = lint("class Foo { constructor() { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "constructor without return should not be flagged"
        );
    }

    #[test]
    fn test_allows_method_return() {
        let diags = lint("class Foo { bar() { return 1; } }");
        assert!(diags.is_empty(), "return in method should not be flagged");
    }

    #[test]
    fn test_flags_nested_return() {
        let diags = lint("class Foo { constructor() { if (true) { return 1; } } }");
        assert_eq!(
            diags.len(),
            1,
            "nested return value in constructor should be flagged"
        );
    }
}
