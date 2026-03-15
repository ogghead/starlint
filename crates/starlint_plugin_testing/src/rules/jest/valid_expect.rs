//! Rule: `jest/valid-expect`
//!
//! Error when `expect()` is called without a matcher (e.g., missing `.toBe()`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/valid-expect";

/// Flags `expect()` calls that are not followed by a matcher method.
#[derive(Debug)]
pub struct ValidExpect;

impl LintRule for ValidExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `expect()` calls to have a corresponding matcher".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        // Check if this is a direct `expect(...)` call (not `expect(...).toBe(...)`)
        let is_expect = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect"
        );

        if !is_expect {
            return;
        }

        // Check if this expect call is used as a standalone expression statement.
        // If so, it means no matcher was chained. We detect this by checking if the
        // source text after the call's closing paren does NOT start with a `.`.
        let source = ctx.source_text();
        let end = usize::try_from(call.span.end).unwrap_or(0);

        // Look at the character(s) right after the call expression span
        let after_call = source.get(end..).unwrap_or("");
        let next_non_ws = after_call.trim_start().chars().next();

        // If the next meaningful character is not `.`, the expect has no matcher
        if next_non_ws != Some('.') {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`expect()` must be followed by a matcher (e.g., `.toBe()`, `.toEqual()`)"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
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

    starlint_rule_framework::lint_rule_test!(ValidExpect);

    #[test]
    fn test_flags_expect_without_matcher() {
        let diags = lint("expect(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect()` without matcher should be flagged"
        );
    }

    #[test]
    fn test_allows_expect_with_matcher() {
        let diags = lint("expect(true).toBe(true);");
        assert!(
            diags.is_empty(),
            "`expect()` with `.toBe()` should not be flagged"
        );
    }

    #[test]
    fn test_allows_expect_to_equal() {
        let diags = lint("expect(1).toEqual(1);");
        assert!(
            diags.is_empty(),
            "`expect()` with `.toEqual()` should not be flagged"
        );
    }
}
