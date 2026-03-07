//! Rule: `jest/require-to-throw-message`
//!
//! Warn when `.toThrow()` or `.toThrowError()` is called without an argument.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/require-to-throw-message";

/// Matcher names that should have an argument.
const THROW_MATCHERS: &[&str] = &["toThrow", "toThrowError"];

/// Flags `.toThrow()` and `.toThrowError()` calls with no arguments.
#[derive(Debug)]
pub struct RequireToThrowMessage;

impl LintRule for RequireToThrowMessage {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `.toThrow()` to have a message argument".to_owned(),
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

        // Match `.toThrow()` or `.toThrowError()` pattern
        let (matcher_name, member_object_id) = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => {
                (member.property.clone(), member.object)
            }
            _ => return,
        };

        if !THROW_MATCHERS.contains(&matcher_name.as_str()) {
            return;
        }

        // Verify it's an expect chain
        let is_expect_chain = is_expect_call_or_chain(member_object_id, ctx);

        if !is_expect_chain {
            return;
        }

        // Flag if no arguments provided
        if call.arguments.is_empty() {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`.{matcher_name}()` should include a message argument to ensure the correct error is thrown"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a node is an `expect(...)` call or chained from one.
fn is_expect_call_or_chain(id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(id) {
        Some(AstNode::CallExpression(call)) => {
            matches!(
                ctx.node(call.callee),
                Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "expect"
            )
        }
        Some(AstNode::StaticMemberExpression(member)) => {
            is_expect_call_or_chain(member.object, ctx)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireToThrowMessage)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_to_throw_without_message() {
        let diags = lint("expect(() => { throw new Error('x'); }).toThrow();");
        assert_eq!(
            diags.len(),
            1,
            "`.toThrow()` without argument should be flagged"
        );
    }

    #[test]
    fn test_flags_to_throw_error_without_message() {
        let diags = lint("expect(fn).toThrowError();");
        assert_eq!(
            diags.len(),
            1,
            "`.toThrowError()` without argument should be flagged"
        );
    }

    #[test]
    fn test_allows_to_throw_with_message() {
        let diags = lint("expect(fn).toThrow('expected error');");
        assert!(
            diags.is_empty(),
            "`.toThrow()` with message should not be flagged"
        );
    }
}
