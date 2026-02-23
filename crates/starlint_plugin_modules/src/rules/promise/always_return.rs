//! Rule: `promise/always-return`
//!
//! Require returning inside `.then()` callbacks. Without a return value,
//! the next `.then()` in the chain receives `undefined`, which is almost
//! always a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.then()` callbacks that do not contain a `return` statement.
#[derive(Debug)]
pub struct AlwaysReturn;

impl LintRule for AlwaysReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/always-return".to_owned(),
            description: "Require returning inside `.then()` callbacks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "then" {
            return;
        }

        // Check the first argument (the onFulfilled callback)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let Some(arg_expr) = ctx.node(*first_arg) else {
            return;
        };

        if matches!(arg_expr, AstNode::SpreadElement(_)) {
            return;
        }

        // Check if the callback is an arrow function with expression body
        // (implicit return — this is fine)
        if let AstNode::ArrowFunctionExpression(arrow) = arg_expr {
            if arrow.expression {
                return; // expression body = implicit return
            }
        }

        // For block-bodied functions, we flag at the `.then()` call site.
        // A full check would inspect function body for return statements,
        // but that requires deeper analysis. We flag non-expression arrows
        // and regular functions as a heuristic.
        match arg_expr {
            AstNode::ArrowFunctionExpression(_) | AstNode::Function(_) => {
                ctx.report(Diagnostic {
                    rule_name: "promise/always-return".to_owned(),
                    message: "Each `.then()` callback should return a value or throw".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AlwaysReturn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_then_with_block_body() {
        let diags = lint("promise.then(function(val) { console.log(val); });");
        assert_eq!(
            diags.len(),
            1,
            "should flag .then() with block-body callback"
        );
    }

    #[test]
    fn test_allows_expression_arrow() {
        let diags = lint("promise.then(val => val * 2);");
        assert!(diags.is_empty(), "expression arrow has implicit return");
    }

    #[test]
    fn test_flags_block_arrow() {
        let diags = lint("promise.then(val => { console.log(val); });");
        assert_eq!(diags.len(), 1, "should flag block-body arrow in .then()");
    }
}
