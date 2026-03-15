//! Rule: `no-useless-return`
//!
//! Disallow redundant `return` statements at the end of a function body.
//! A bare `return;` at the end of a function is unnecessary.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags redundant `return;` statements at the end of function bodies.
#[derive(Debug)]
pub struct NoUselessReturn;

impl LintRule for NoUselessReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-return".to_owned(),
            description: "Disallow redundant return statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::FunctionBody])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::FunctionBody(body) = node else {
            return;
        };

        // Check if the last statement is a bare return (no argument)
        let Some(&last_id) = body.statements.last() else {
            return;
        };

        // Extract span from the resolved node before mutably borrowing ctx
        let ret_span = match ctx.node(last_id) {
            Some(AstNode::ReturnStatement(ret)) if ret.argument.is_none() => {
                Span::new(ret.span.start, ret.span.end)
            }
            _ => return,
        };

        ctx.report(Diagnostic {
            rule_name: "no-useless-return".to_owned(),
            message: "Unnecessary return statement".to_owned(),
            span: ret_span,
            severity: Severity::Warning,
            help: Some("Remove the unnecessary `return`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove unnecessary `return`".to_owned(),
                edits: vec![Edit {
                    span: ret_span,
                    replacement: String::new(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUselessReturn);

    #[test]
    fn test_flags_trailing_return() {
        let diags = lint("function foo() { doSomething(); return; }");
        assert_eq!(diags.len(), 1, "trailing bare return should be flagged");
    }

    #[test]
    fn test_allows_return_with_value() {
        let diags = lint("function foo() { return 42; }");
        assert!(diags.is_empty(), "return with value should not be flagged");
    }

    #[test]
    fn test_allows_no_return() {
        let diags = lint("function foo() { doSomething(); }");
        assert!(
            diags.is_empty(),
            "function without return should not be flagged"
        );
    }
}
