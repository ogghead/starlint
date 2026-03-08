//! Rule: `no-return-assign`
//!
//! Disallow assignment operators in `return` statements. Using assignment
//! in a return is often a mistake (intended `===` comparison).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags assignment expressions in `return` statements.
#[derive(Debug)]
pub struct NoReturnAssign;

impl LintRule for NoReturnAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-return-assign".to_owned(),
            description: "Disallow assignment operators in `return` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ReturnStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ReturnStatement(ret) = node else {
            return;
        };

        let Some(arg_id) = ret.argument else {
            return;
        };

        if contains_assignment(arg_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: "no-return-assign".to_owned(),
                message: "Assignment in return statement — use a separate statement or `===` for comparison".to_owned(),
                span: Span::new(ret.span.start, ret.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression contains an assignment operator.
fn contains_assignment(id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(id) {
        Some(AstNode::AssignmentExpression(_)) => true,
        // ParenthesizedExpression is not modeled in the flat AST;
        // the parser unwraps parentheses.
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoReturnAssign)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_return_assignment() {
        let diags = lint("function f() { return x = 1; }");
        assert_eq!(diags.len(), 1, "return with assignment should be flagged");
    }

    #[test]
    fn test_flags_parenthesized_assignment() {
        let diags = lint("function f() { return (x = 1); }");
        assert_eq!(
            diags.len(),
            1,
            "return with parenthesized assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_return_value() {
        let diags = lint("function f() { return x + 1; }");
        assert!(
            diags.is_empty(),
            "return with expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_comparison() {
        let diags = lint("function f() { return x === 1; }");
        assert!(
            diags.is_empty(),
            "return with comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("function f() { return; }");
        assert!(diags.is_empty(), "empty return should not be flagged");
    }
}
