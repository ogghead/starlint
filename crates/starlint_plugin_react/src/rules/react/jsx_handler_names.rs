//! Rule: `react/jsx-handler-names`
//!
//! Suggest that event handler props (`onClick`, `onChange`, etc.) should
//! reference handler functions starting with `handle`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-handler-names";

/// Suggests that event handler props (names starting with `on`) should reference
/// handler functions named with the `handle` prefix.
#[derive(Debug)]
pub struct JsxHandlerNames;

impl LintRule for JsxHandlerNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce handler function naming conventions for event props".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXAttribute(attr) = node else {
            return;
        };

        // Check if the prop name starts with "on" followed by an uppercase letter
        let prop_name = attr.name.as_str();

        if !prop_name.starts_with("on") {
            return;
        }

        // Make sure the char after "on" is uppercase (e.g., onClick, onChange)
        let Some(after_on) = prop_name.get(2..3) else {
            return;
        };
        if !after_on
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase())
        {
            return;
        }

        // Check if the value is an expression container containing an identifier reference
        let Some(value_id) = attr.value else {
            return;
        };

        let Some(AstNode::JSXExpressionContainer(container)) = ctx.node(value_id) else {
            return;
        };

        let Some(expr_id) = container.expression else {
            return;
        };

        if let Some(AstNode::IdentifierReference(ident_ref)) = ctx.node(expr_id) {
            let handler_name = ident_ref.name.as_str();
            // The handler should start with "handle" or "on" (passing props through)
            if !handler_name.starts_with("handle") && !handler_name.starts_with("on") {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Handler function for `{prop_name}` should be named starting with `handle` (e.g., `handle{}`)",
                        &prop_name[2..]
                    ),
                    span: Span::new(attr.span.start, attr.span.end),
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

    starlint_rule_framework::lint_rule_test!(JsxHandlerNames);

    #[test]
    fn test_flags_non_handle_prefix() {
        let diags = lint("const el = <button onClick={doSomething} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag handler not starting with 'handle'"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_handle_prefix() {
        let diags = lint("const el = <button onClick={handleClick} />;");
        assert!(
            diags.is_empty(),
            "should not flag handler starting with 'handle'"
        );
    }

    #[test]
    fn test_allows_on_prefix_passthrough() {
        let diags = lint("const el = <button onClick={onClick} />;");
        assert!(
            diags.is_empty(),
            "should not flag handler starting with 'on' (prop passthrough)"
        );
    }
}
