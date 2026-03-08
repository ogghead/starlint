//! Rule: `promise/avoid-new`
//!
//! Forbid creating `new Promise`. Encourages use of utility functions
//! like `Promise.resolve()`, `Promise.reject()`, or async functions
//! instead of the Promise constructor.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Promise(...)` constructor calls.
#[derive(Debug)]
pub struct AvoidNew;

impl LintRule for AvoidNew {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/avoid-new".to_owned(),
            description: "Forbid creating `new Promise`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let is_promise = matches!(
            ctx.node(new_expr.callee),
            Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "Promise"
        );

        if is_promise {
            ctx.report(Diagnostic {
                rule_name: "promise/avoid-new".to_owned(),
                message: "Avoid creating `new Promise` — prefer async functions or `Promise.resolve()`/`Promise.reject()`".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AvoidNew)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_promise() {
        let diags = lint("const p = new Promise((resolve) => resolve(1));");
        assert_eq!(diags.len(), 1, "should flag new Promise");
    }

    #[test]
    fn test_allows_promise_resolve() {
        let diags = lint("const p = Promise.resolve(1);");
        assert!(diags.is_empty(), "Promise.resolve should be allowed");
    }

    #[test]
    fn test_allows_other_new() {
        let diags = lint("const m = new Map();");
        assert!(diags.is_empty(), "new Map should not be flagged");
    }
}
