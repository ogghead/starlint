//! Rule: `typescript/no-unnecessary-condition`
//!
//! Disallow unnecessary conditions. Flags `if (true)`, `if (false)`, and
//! `while (true)` where the condition is a boolean literal and is therefore
//! always known at compile time.
//!
//! Simplified syntax-only version — full checking requires type information.
//! The full rule also flags conditions whose type is always truthy/falsy
//! after narrowing; this simplified version only detects boolean literal
//! constants.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unnecessary-condition";

/// Flags `if` and `while` statements whose condition is a boolean literal.
#[derive(Debug)]
pub struct NoUnnecessaryCondition;

impl LintRule for NoUnnecessaryCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary conditions (boolean literal in condition position)"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement, AstNodeType::WhileStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::IfStatement(stmt) => {
                if let Some(value) = boolean_literal_value(ctx, stmt.test) {
                    let label = if value { "true" } else { "false" };
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "Unnecessary condition — `if ({label})` is always {}, \
                             the branch is {}",
                            label,
                            if value { "always taken" } else { "dead code" },
                        ),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::WhileStatement(stmt) => {
                if let Some(value) = boolean_literal_value(ctx, stmt.test) {
                    let label = if value { "true" } else { "false" };
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "Unnecessary condition — `while ({label})` is a constant \
                             loop condition"
                        ),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// If the expression is a `BooleanLiteral`, return its value.
fn boolean_literal_value(ctx: &LintContext<'_>, id: NodeId) -> Option<bool> {
    if let Some(AstNode::BooleanLiteral(lit)) = ctx.node(id) {
        Some(lit.value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnnecessaryCondition, "test.ts");

    #[test]
    fn test_flags_if_true() {
        let diags = lint("if (true) { console.log('always'); }");
        assert_eq!(diags.len(), 1, "`if (true)` should be flagged");
    }

    #[test]
    fn test_flags_if_false() {
        let diags = lint("if (false) { console.log('never'); }");
        assert_eq!(diags.len(), 1, "`if (false)` should be flagged");
    }

    #[test]
    fn test_flags_while_true() {
        let diags = lint("while (true) { break; }");
        assert_eq!(diags.len(), 1, "`while (true)` should be flagged");
    }

    #[test]
    fn test_allows_dynamic_condition() {
        let diags = lint("const x = Math.random(); if (x > 0.5) { console.log('maybe'); }");
        assert!(diags.is_empty(), "dynamic condition should not be flagged");
    }

    #[test]
    fn test_allows_variable_while_condition() {
        let diags = lint("let running = true; while (running) { running = false; }");
        assert!(
            diags.is_empty(),
            "variable in while condition should not be flagged"
        );
    }
}
