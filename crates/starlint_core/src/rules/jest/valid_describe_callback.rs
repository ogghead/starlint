//! Rule: `jest/valid-describe-callback`
//!
//! Error when `describe` callback is async or returns a value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/valid-describe-callback";

/// Flags `describe` blocks with async callbacks or return values.
#[derive(Debug)]
pub struct ValidDescribeCallback;

impl LintRule for ValidDescribeCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow async `describe` callbacks and return values".to_owned(),
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

        // Check callee is `describe`
        let is_describe = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "describe"
        );

        if !is_describe {
            return;
        }

        // The callback is the second argument
        let Some(callback_id) = call.arguments.get(1) else {
            return;
        };

        // Extract needed data before calling ctx.report() to avoid borrow conflict
        let callback_info = match ctx.node(*callback_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => {
                Some((arrow.span, arrow.is_async, arrow.expression, true))
            }
            Some(AstNode::Function(func)) => Some((func.span, func.is_async, false, false)),
            _ => None,
        };

        if let Some((span, is_async, is_expression, is_arrow)) = callback_info {
            if is_async {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "`describe` callback must not be async".to_owned(),
                    span: Span::new(span.start, span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
            // Check for expression body (implicit return) - only for arrow functions
            if is_arrow && is_expression {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "`describe` callback must not return a value".to_owned(),
                    span: Span::new(span.start, span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ValidDescribeCallback)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_async_describe() {
        let diags = lint("describe('suite', async () => {});");
        assert_eq!(diags.len(), 1, "async describe callback should be flagged");
    }

    #[test]
    fn test_flags_async_function_describe() {
        let diags = lint("describe('suite', async function() {});");
        assert_eq!(
            diags.len(),
            1,
            "async function describe callback should be flagged"
        );
    }

    #[test]
    fn test_allows_sync_describe() {
        let diags = lint("describe('suite', () => { it('works', () => {}); });");
        assert!(
            diags.is_empty(),
            "sync describe callback should not be flagged"
        );
    }
}
