//! Rule: `promise/catch-or-return`
//!
//! Require `.catch()` or return for promises. Ensures that promise chains
//! either handle errors via `.catch()` or are returned to the caller.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.then()` calls in expression statements without a trailing `.catch()`.
#[derive(Debug)]
pub struct CatchOrReturn;

impl LintRule for CatchOrReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/catch-or-return".to_owned(),
            description: "Require `.catch()` or return for promises".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExpressionStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // We match on ExpressionStatement to ensure the promise chain is
        // a top-level statement (not returned or assigned).
        let AstNode::ExpressionStatement(stmt) = node else {
            return;
        };

        let method = {
            let Some(AstNode::CallExpression(call)) = ctx.node(stmt.expression) else {
                return;
            };
            let callee_id = call.callee;
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(callee_id) else {
                return;
            };
            member.property.clone()
        };

        // If the outermost call is .catch() or .finally(), that's fine
        if method == "catch" || method == "finally" {
            return;
        }

        // If the outermost call is .then(), flag it
        if method == "then" {
            ctx.report(Diagnostic {
                rule_name: "promise/catch-or-return".to_owned(),
                message: "Promise chain must end with `.catch()` or be returned".to_owned(),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Error,
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CatchOrReturn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_then_without_catch() {
        let diags = lint("promise.then(val => val);");
        assert_eq!(diags.len(), 1, "should flag .then() without .catch()");
    }

    #[test]
    fn test_allows_then_with_catch() {
        let diags = lint("promise.then(val => val).catch(err => err);");
        assert!(diags.is_empty(), ".then().catch() should be allowed");
    }

    #[test]
    fn test_allows_catch_only() {
        let diags = lint("promise.catch(err => console.error(err));");
        assert!(diags.is_empty(), ".catch() alone should be allowed");
    }
}
