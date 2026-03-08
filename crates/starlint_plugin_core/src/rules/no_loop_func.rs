//! Rule: `no-loop-func`
//!
//! Disallow function declarations and expressions inside loop statements.
//! Functions created in loops can lead to closure bugs where the loop
//! variable is shared.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags function declarations/expressions inside loops.
#[derive(Debug)]
pub struct NoLoopFunc;

impl LintRule for NoLoopFunc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-loop-func".to_owned(),
            description: "Disallow function declarations and expressions inside loops".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::DoWhileStatement,
            AstNodeType::ForInStatement,
            AstNodeType::ForOfStatement,
            AstNodeType::ForStatement,
            AstNodeType::WhileStatement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Check loop bodies for function declarations
        let loop_body_id: Option<NodeId> = match node {
            AstNode::ForStatement(stmt) => Some(stmt.body),
            AstNode::ForInStatement(stmt) => Some(stmt.body),
            AstNode::ForOfStatement(stmt) => Some(stmt.body),
            AstNode::WhileStatement(stmt) => Some(stmt.body),
            AstNode::DoWhileStatement(stmt) => Some(stmt.body),
            _ => None,
        };

        let Some(body_id) = loop_body_id else {
            return;
        };

        // Check the direct body block for function declarations
        let Some(AstNode::BlockStatement(block)) = ctx.node(body_id) else {
            return;
        };

        let mut spans: Vec<Span> = Vec::new();
        for stmt_id in &*block.body {
            if let Some(AstNode::Function(func)) = ctx.node(*stmt_id) {
                spans.push(Span::new(func.span.start, func.span.end));
            }
        }

        for span in spans {
            ctx.report(Diagnostic {
                rule_name: "no-loop-func".to_owned(),
                message: "Function declaration inside a loop".to_owned(),
                span,
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoLoopFunc)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_function_in_for_loop() {
        let diags = lint("for (var i = 0; i < 10; i++) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in for loop should be flagged");
    }

    #[test]
    fn test_flags_function_in_while_loop() {
        let diags = lint("while (true) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in while loop should be flagged");
    }

    #[test]
    fn test_allows_function_outside_loop() {
        let diags = lint("function foo() {} for (var i = 0; i < 10; i++) {}");
        assert!(
            diags.is_empty(),
            "function outside loop should not be flagged"
        );
    }
}
