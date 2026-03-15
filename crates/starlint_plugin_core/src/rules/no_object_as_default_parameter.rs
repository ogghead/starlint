//! Rule: `no-object-as-default-parameter`
//!
//! Disallow using object literals as default parameter values. A mutable object
//! literal in a default parameter creates a new object on every call, which can
//! be confusing and wasteful. Prefer destructuring defaults instead:
//! `function foo({ a = 1 } = {})` rather than `function foo(x = { a: 1 })`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags object literals used as default parameter values.
#[derive(Debug)]
pub struct NoObjectAsDefaultParameter;

impl LintRule for NoObjectAsDefaultParameter {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-object-as-default-parameter".to_owned(),
            description: "Disallow object literals as default parameter values".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let params = match node {
            AstNode::Function(f) => &f.params,
            AstNode::ArrowFunctionExpression(arrow) => &arrow.params,
            _ => return,
        };

        // Iterate param NodeIds. Default params produce AssignmentPattern nodes
        // with `left` = binding and `right` = default value expression.
        for param_id in &**params {
            let Some(AstNode::AssignmentPattern(assign)) = ctx.node(*param_id) else {
                continue;
            };

            // Check if the default value (right side) is an object expression
            let Some(AstNode::ObjectExpression(obj)) = ctx.node(assign.right) else {
                continue;
            };

            if obj.properties.is_empty() {
                continue;
            }

            let assign_span = Span::new(assign.span.start, assign.span.end);
            ctx.report(Diagnostic {
                rule_name: "no-object-as-default-parameter".to_owned(),
                message: "Do not use an object literal as a default parameter — prefer destructuring defaults".to_owned(),
                span: assign_span,
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

    starlint_rule_framework::lint_rule_test!(NoObjectAsDefaultParameter);

    #[test]
    fn test_flags_object_literal_default() {
        let diags = lint("function foo(x = { a: 1 }) {}");
        assert_eq!(
            diags.len(),
            1,
            "object literal as default param should be flagged"
        );
    }

    #[test]
    fn test_flags_object_literal_default_multiple_props() {
        let diags = lint("function foo(x = { a: 1, b: 2 }) {}");
        assert_eq!(
            diags.len(),
            1,
            "object literal with multiple props as default should be flagged"
        );
    }

    #[test]
    fn test_allows_array_default() {
        let diags = lint("function foo(x = []) {}");
        assert!(diags.is_empty(), "array default should not be flagged");
    }

    #[test]
    fn test_allows_string_default() {
        let diags = lint("function foo(x = 'default') {}");
        assert!(diags.is_empty(), "string default should not be flagged");
    }

    #[test]
    fn test_allows_empty_object_for_destructured_param() {
        let diags = lint("function foo({ a = 1 } = {}) {}");
        assert!(
            diags.is_empty(),
            "empty object as default for destructured param should not be flagged"
        );
    }

    #[test]
    fn test_allows_numeric_default() {
        let diags = lint("function foo(x = 42) {}");
        assert!(diags.is_empty(), "numeric default should not be flagged");
    }

    #[test]
    fn test_allows_null_default() {
        let diags = lint("function foo(x = null) {}");
        assert!(diags.is_empty(), "null default should not be flagged");
    }

    #[test]
    fn test_flags_arrow_function_object_default() {
        let diags = lint("const foo = (x = { a: 1 }) => {};");
        assert_eq!(
            diags.len(),
            1,
            "object literal default in arrow function should be flagged"
        );
    }
}
