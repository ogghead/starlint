//! Rule: `jest/no-restricted-matchers`
//!
//! Warn when restricted matchers are used (e.g., `.toBeTruthy()`, `.toBeFalsy()`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-restricted-matchers";

/// Default restricted matchers. These are commonly flagged because they produce
/// less informative test failures compared to explicit matchers.
const RESTRICTED_MATCHERS: &[&str] = &[
    "toBeTruthy",
    "toBeFalsy",
    "resolves",
    "rejects",
    "toMatchSnapshot",
];

/// Flags usage of restricted Jest matchers in expect chains.
#[derive(Debug)]
pub struct NoRestrictedMatchers;

impl LintRule for NoRestrictedMatchers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow restricted Jest matchers".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("expect(") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Match `expect(...).matcher(...)` or `expect(...).not.matcher(...)` or
        // `expect(...).resolves/rejects`
        let matcher_name = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => member.property.clone(),
            _ => return,
        };

        if !RESTRICTED_MATCHERS.contains(&matcher_name.as_str()) {
            return;
        }

        // Verify this is part of an expect chain by walking up the member expression
        let is_expect_chain = is_in_expect_chain(call.callee, ctx);

        if is_expect_chain {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`.{matcher_name}` matcher is restricted — use a more specific matcher"
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

/// Check whether a callee expression is part of an `expect(...)` chain.
/// Walks through member expression objects looking for `expect(...)`.
fn is_in_expect_chain(id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(id) {
        Some(AstNode::StaticMemberExpression(member)) => {
            is_expect_call_or_chain(member.object, ctx)
        }
        _ => false,
    }
}

/// Recursively check if an expression is `expect(...)` or a chain from it.
fn is_expect_call_or_chain(id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(id) {
        Some(AstNode::CallExpression(call)) => {
            // Direct `expect(...)` call
            matches!(
                ctx.node(call.callee),
                Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "expect"
            )
        }
        Some(AstNode::StaticMemberExpression(member)) => {
            // `expect(...).not` or `expect(...).resolves` etc.
            is_expect_call_or_chain(member.object, ctx)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRestrictedMatchers)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_to_be_truthy() {
        let diags = lint("expect(value).toBeTruthy();");
        assert_eq!(
            diags.len(),
            1,
            "`.toBeTruthy()` should be flagged as restricted"
        );
    }

    #[test]
    fn test_flags_to_be_falsy() {
        let diags = lint("expect(value).toBeFalsy();");
        assert_eq!(
            diags.len(),
            1,
            "`.toBeFalsy()` should be flagged as restricted"
        );
    }

    #[test]
    fn test_allows_to_be() {
        let diags = lint("expect(value).toBe(true);");
        assert!(
            diags.is_empty(),
            "`.toBe()` should not be flagged as it is not restricted"
        );
    }
}
