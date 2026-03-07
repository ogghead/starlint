//! Rule: `promise/no-return-wrap`
//!
//! Forbid wrapping return values in `Promise.resolve()` or `Promise.reject()`
//! inside `.then()` and `.catch()` handlers. These handlers already wrap
//! return values in promises automatically.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `Promise.resolve(val)` or `Promise.reject(err)` used inside
/// `.then()` or `.catch()` callback arguments.
///
/// Heuristic: scans the source text of callback arguments for these patterns.
#[derive(Debug)]
pub struct NoReturnWrap;

impl LintRule for NoReturnWrap {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-return-wrap".to_owned(),
            description: "Forbid wrapping return values in `Promise.resolve`/`Promise.reject`"
                .to_owned(),
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

        let method = member.property.as_str();
        if method != "then" && method != "catch" {
            return;
        }

        // Check callback arguments for Promise.resolve/reject patterns
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

            if body_text.contains("Promise.resolve(") || body_text.contains("Promise.reject(") {
                ctx.report(Diagnostic {
                    rule_name: "promise/no-return-wrap".to_owned(),
                    message: format!(
                        "Unnecessary `Promise.resolve`/`Promise.reject` in `.{method}()` — return the value directly"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoReturnWrap)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_promise_resolve_in_then() {
        let diags = lint("p.then(val => Promise.resolve(val));");
        assert_eq!(diags.len(), 1, "should flag Promise.resolve in .then()");
    }

    #[test]
    fn test_flags_promise_reject_in_catch() {
        let diags = lint("p.catch(err => Promise.reject(err));");
        assert_eq!(diags.len(), 1, "should flag Promise.reject in .catch()");
    }

    #[test]
    fn test_allows_direct_return() {
        let diags = lint("p.then(val => val * 2);");
        assert!(diags.is_empty(), "direct return should not be flagged");
    }
}
