//! Rule: `promise/no-nesting`
//!
//! Forbid nesting `.then()` or `.catch()` inside another `.then()`/`.catch()`.
//! Nested promise chains flatten poorly and should be refactored to chained
//! `.then()` calls or `async`/`await`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.then()` or `.catch()` calls whose callee object is itself
/// a `.then()` or `.catch()` call inside an argument position, detected
/// by scanning the source text of callback arguments.
#[derive(Debug)]
pub struct NoNesting;

impl LintRule for NoNesting {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-nesting".to_owned(),
            description: "Forbid nesting `.then()`/`.catch()` chains".to_owned(),
            category: Category::Style,
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method = member.property.as_str();
        if method != "then" && method != "catch" {
            return;
        }

        // Check each argument for nested .then()/.catch() patterns
        for arg in &call.arguments {
            let Some(arg_expr) = ctx.node(*arg) else {
                continue;
            };

            if matches!(arg_expr, AstNode::SpreadElement(_)) {
                continue;
            }

            let start = usize::try_from(arg_expr.span().start).unwrap_or(0);
            let end = usize::try_from(arg_expr.span().end).unwrap_or(0);
            let body_text = ctx.source_text().get(start..end).unwrap_or_default();

            if body_text.contains(".then(") || body_text.contains(".catch(") {
                ctx.report(Diagnostic {
                    rule_name: "promise/no-nesting".to_owned(),
                    message: "Avoid nesting `.then()`/`.catch()` — flatten the chain or use `async`/`await`".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                return; // Only report once per call
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNesting)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_nested_then() {
        let diags = lint("p.then(val => val.then(x => x));");
        assert!(!diags.is_empty(), "should flag nested .then()");
    }

    #[test]
    fn test_flags_nested_catch_in_then() {
        let diags = lint("p.then(val => other.catch(e => e));");
        assert!(!diags.is_empty(), "should flag nested .catch() in .then()");
    }

    #[test]
    fn test_allows_flat_chain() {
        let diags = lint("p.then(val => val * 2).catch(err => err);");
        assert!(diags.is_empty(), "flat chain should not be flagged");
    }
}
