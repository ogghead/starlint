//! Rule: `jest/no-restricted-jest-methods`
//!
//! Warn when restricted Jest methods are used (e.g., `jest.advanceTimersByTime`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-restricted-jest-methods";

/// Default restricted `jest.*` methods.
const RESTRICTED_METHODS: &[&str] = &[
    "advanceTimersByTime",
    "advanceTimersByTimeAsync",
    "advanceTimersToNextTimer",
    "advanceTimersToNextTimerAsync",
    "clearAllTimers",
    "retryTimes",
];

/// Flags usage of restricted `jest.*` methods.
#[derive(Debug)]
pub struct NoRestrictedJestMethods;

impl LintRule for NoRestrictedJestMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow restricted Jest methods".to_owned(),
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

        // Match `jest.<method>(...)` pattern
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if !matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "jest")
        {
            return;
        }

        let method_name = member.property.as_str();
        if RESTRICTED_METHODS.contains(&method_name) {
            let method_name_owned = method_name.to_owned();
            let call_span = Span::new(call.span.start, call.span.end);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`jest.{method_name_owned}` is restricted"),
                span: call_span,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRestrictedJestMethods)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_restricted_method() {
        let diags = lint("jest.advanceTimersByTime(1000);");
        assert_eq!(
            diags.len(),
            1,
            "`jest.advanceTimersByTime` should be flagged"
        );
    }

    #[test]
    fn test_flags_retry_times() {
        let diags = lint("jest.retryTimes(3);");
        assert_eq!(diags.len(), 1, "`jest.retryTimes` should be flagged");
    }

    #[test]
    fn test_allows_unrestricted_method() {
        let diags = lint("jest.fn();");
        assert!(
            diags.is_empty(),
            "`jest.fn` should not be flagged as it is not restricted"
        );
    }
}
