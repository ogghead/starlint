//! Rule: `promise/valid-params`
//!
//! Enforce correct number of parameters to Promise static methods.
//! `Promise.resolve()` takes 0-1 args, `Promise.reject()` takes 0-1 args,
//! `Promise.all()` takes exactly 1 arg, etc.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `Promise.all()`, `Promise.race()`, `Promise.allSettled()`,
/// and `Promise.any()` called with incorrect argument counts.
#[derive(Debug)]
pub struct ValidParams;

impl LintRule for ValidParams {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/valid-params".to_owned(),
            description: "Enforce correct number of params to Promise methods".to_owned(),
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

        let method = {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
                return;
            };
            let is_promise = matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "Promise");
            if !is_promise {
                return;
            }
            member.property.clone()
        };
        let arg_count = call.arguments.len();

        let expected = match method.as_str() {
            // These require exactly 1 argument (an iterable)
            "all" | "allSettled" | "any" | "race" => Some((1, 1)),
            // These take 0 or 1 argument
            "resolve" | "reject" => Some((0, 1)),
            _ => None,
        };

        if let Some((min, max)) = expected {
            if arg_count < min || arg_count > max {
                let expected_msg = if min == max {
                    format!("exactly {min}")
                } else {
                    format!("{min}-{max}")
                };
                ctx.report(Diagnostic {
                    rule_name: "promise/valid-params".to_owned(),
                    message: format!(
                        "`Promise.{method}()` expects {expected_msg} argument(s), got {arg_count}"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ValidParams)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_promise_all_no_args() {
        let diags = lint("Promise.all();");
        assert_eq!(diags.len(), 1, "should flag Promise.all() with no args");
    }

    #[test]
    fn test_flags_promise_resolve_two_args() {
        let diags = lint("Promise.resolve(1, 2);");
        assert_eq!(diags.len(), 1, "should flag Promise.resolve with 2 args");
    }

    #[test]
    fn test_allows_promise_all_one_arg() {
        let diags = lint("Promise.all([p1, p2]);");
        assert!(diags.is_empty(), "Promise.all with 1 arg should be allowed");
    }
}
