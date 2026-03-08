//! Rule: `jest/valid-title`
//!
//! Warn when `describe`/`it`/`test` titles are empty strings or not string literals.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/valid-title";

/// Test block names to check.
const TEST_BLOCKS: &[&str] = &["describe", "it", "test"];

/// Flags `describe`/`it`/`test` calls with empty or non-string titles.
#[derive(Debug)]
pub struct ValidTitle;

impl LintRule for ValidTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require valid titles for `describe`/`it`/`test` blocks".to_owned(),
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

        // Check if callee is describe/it/test (direct identifier)
        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str(),
            _ => return,
        };

        if !TEST_BLOCKS.contains(&callee_name) {
            return;
        }

        // Check the first argument (the title)
        let Some(first_arg_id) = call.arguments.first() else {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`{callee_name}()` must have a title as its first argument"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
            return;
        };

        match ctx.node(*first_arg_id) {
            Some(AstNode::StringLiteral(lit)) => {
                if lit.value.is_empty() {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!("`{callee_name}()` title must not be empty"),
                        span: Span::new(lit.span.start, lit.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            Some(AstNode::TemplateLiteral(_)) => {
                // Template literals are acceptable
            }
            _ => {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!("`{callee_name}()` title must be a string literal"),
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

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ValidTitle)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_title() {
        let diags = lint("describe('', () => {});");
        assert_eq!(diags.len(), 1, "empty describe title should be flagged");
    }

    #[test]
    fn test_flags_non_string_title() {
        let diags = lint("it(123, () => {});");
        assert_eq!(diags.len(), 1, "non-string title should be flagged");
    }

    #[test]
    fn test_allows_valid_title() {
        let diags = lint("test('should work', () => {});");
        assert!(diags.is_empty(), "valid string title should not be flagged");
    }
}
