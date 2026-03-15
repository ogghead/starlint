//! Rule: `jsx-a11y/no-static-element-interactions`
//!
//! Forbid event handlers on static elements (`<div>`, `<span>`, etc.) without a role.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-static-element-interactions";

/// Static (non-interactive) HTML elements.
const STATIC_ELEMENTS: &[&str] = &[
    "div",
    "span",
    "section",
    "article",
    "aside",
    "footer",
    "header",
    "main",
    "nav",
    "p",
    "blockquote",
    "pre",
    "figure",
    "figcaption",
    "dd",
    "dl",
    "dt",
    "ul",
    "ol",
    "li",
    "fieldset",
    "table",
    "tbody",
    "thead",
    "tfoot",
    "tr",
    "td",
    "th",
];

/// Event handler attribute names that indicate interactivity.
const EVENT_HANDLERS: &[&str] = &[
    "onClick",
    "onKeyDown",
    "onKeyUp",
    "onKeyPress",
    "onMouseDown",
    "onMouseUp",
    "onDoubleClick",
];

#[derive(Debug)]
pub struct NoStaticElementInteractions;

/// Check if an attribute exists on a JSX element.
fn has_attribute(
    opening: &starlint_ast::node::JSXOpeningElementNode,
    name: &str,
    ctx: &LintContext<'_>,
) -> bool {
    opening.attributes.iter().any(|attr_id| {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
            attr.name.as_str() == name
        } else {
            false
        }
    })
}

impl LintRule for NoStaticElementInteractions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid event handlers on static elements without a role".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        let element_name = opening.name.as_str();

        if !STATIC_ELEMENTS.contains(&element_name) {
            return;
        }

        // If it has a role, it is intentionally interactive
        if has_attribute(opening, "role", ctx) {
            return;
        }

        // Check for event handler attributes
        let has_event_handler = EVENT_HANDLERS
            .iter()
            .any(|handler| has_attribute(opening, handler, ctx));

        if has_event_handler {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`<{element_name}>` with event handlers must have a `role` attribute"
                ),
                span: Span::new(opening.span.start, opening.span.end),
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

    starlint_rule_framework::lint_rule_test!(NoStaticElementInteractions);

    #[test]
    fn test_flags_div_with_onclick_no_role() {
        let diags = lint(r"const el = <div onClick={handleClick}>content</div>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_div_with_onclick_and_role() {
        let diags = lint(r#"const el = <div onClick={handleClick} role="button">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_button_with_onclick() {
        let diags = lint(r"const el = <button onClick={handleClick}>click</button>;");
        assert!(diags.is_empty());
    }
}
