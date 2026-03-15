//! Rule: `no-new-func`
//!
//! Disallow `new Function()`. The `Function` constructor creates functions
//! from strings, similar to `eval()`, and carries the same security risks.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Function()` and `Function()` constructor calls.
#[derive(Debug)]
pub struct NoNewFunc;

impl LintRule for NoNewFunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-func".to_owned(),
            description: "Disallow `new Function()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression, AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (callee_id, span) = match node {
            AstNode::NewExpression(new_expr) => (new_expr.callee, new_expr.span),
            AstNode::CallExpression(call) => (call.callee, call.span),
            _ => return,
        };

        let is_function_constructor = matches!(
            ctx.node(callee_id),
            Some(AstNode::IdentifierReference(id)) if id.name == "Function"
        );

        if is_function_constructor {
            ctx.report(Diagnostic {
                rule_name: "no-new-func".to_owned(),
                message: "The `Function` constructor is `eval`".to_owned(),
                span: Span::new(span.start, span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNewFunc);

    #[test]
    fn test_flags_new_function() {
        let diags = lint("var f = new Function('a', 'return a');");
        assert_eq!(diags.len(), 1, "new Function() should be flagged");
    }

    #[test]
    fn test_flags_function_call() {
        let diags = lint("var f = Function('a', 'return a');");
        assert_eq!(diags.len(), 1, "Function() call should be flagged");
    }

    #[test]
    fn test_allows_normal_constructor() {
        let diags = lint("var x = new MyClass();");
        assert!(diags.is_empty(), "normal constructor should not be flagged");
    }
}
