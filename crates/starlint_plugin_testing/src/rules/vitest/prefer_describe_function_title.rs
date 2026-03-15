//! Rule: `vitest/prefer-describe-function-title`
//!
//! Suggest that `describe` block titles reference the function being tested.
//! When a `describe` block wraps tests for a specific function, its title
//! should match the function name for discoverability and organization.
//! This rule flags `describe` calls where the title is an empty string.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-describe-function-title";

/// Suggest meaningful `describe` block titles.
#[derive(Debug)]
pub struct PreferDescribeFunctionTitle;

impl LintRule for PreferDescribeFunctionTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce meaningful `describe` block titles".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("describe(") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Match `describe(...)` calls.
        let is_describe = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str() == "describe",
            _ => false,
        };

        if !is_describe {
            return;
        }

        // Check the first argument (the title).
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Flag empty string titles.
        if let Some(AstNode::StringLiteral(lit)) = ctx.node(*first_arg) {
            if lit.value.as_str().trim().is_empty() {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "`describe` block should have a meaningful title — use the function name or feature being tested".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }

        // Flag template literals with no expressions that are empty.
        if let Some(AstNode::TemplateLiteral(tpl)) = ctx.node(*first_arg) {
            if tpl.expressions.is_empty() {
                let is_empty = tpl.quasis.iter().all(|q| q.trim().is_empty());
                if is_empty {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "`describe` block should have a meaningful title — use the function name or feature being tested".to_owned(),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferDescribeFunctionTitle);

    #[test]
    fn test_flags_empty_describe_title() {
        let source = r#"describe("", () => {});"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`describe` with empty title should be flagged"
        );
    }

    #[test]
    fn test_allows_meaningful_describe_title() {
        let source = r#"describe("calculateTotal", () => {});"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`describe` with meaningful title should not be flagged"
        );
    }

    #[test]
    fn test_flags_whitespace_only_describe_title() {
        let source = r#"describe("  ", () => {});"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`describe` with whitespace-only title should be flagged"
        );
    }
}
