//! Rule: `promise/prefer-await-to-then`
//!
//! Prefer `async`/`await` over `.then()` chains. Modern async syntax
//! is generally more readable and easier to debug.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.then()` calls, suggesting `async`/`await` instead.
#[derive(Debug)]
pub struct PreferAwaitToThen;

impl LintRule for PreferAwaitToThen {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/prefer-await-to-then".to_owned(),
            description: "Prefer `async`/`await` over `.then()` chains".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let is_then = matches!(
            ctx.node(call.callee),
            Some(AstNode::StaticMemberExpression(member)) if member.property.as_str() == "then"
        );

        if is_then {
            ctx.report(Diagnostic {
                rule_name: "promise/prefer-await-to-then".to_owned(),
                message: "Prefer `async`/`await` over `.then()` chains".to_owned(),
                span: Span::new(call.span.start, call.span.end),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferAwaitToThen)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_then_usage() {
        let diags = lint("promise.then(val => console.log(val));");
        assert_eq!(diags.len(), 1, "should flag .then() usage");
    }

    #[test]
    fn test_allows_await() {
        let diags = lint("async function f() { const val = await promise; }");
        assert!(diags.is_empty(), "await should not be flagged");
    }

    #[test]
    fn test_flags_chained_then() {
        let diags = lint("p.then(a => a).then(b => b);");
        assert_eq!(diags.len(), 2, "should flag both .then() calls");
    }
}
