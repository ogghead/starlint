//! Rule: `jsx-a11y/mouse-events-have-key-events`
//!
//! Enforce `onMouseOver`/`onMouseOut` have `onFocus`/`onBlur`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/mouse-events-have-key-events";

#[derive(Debug)]
pub struct MouseEventsHaveKeyEvents;

/// Check if an attribute with the given name exists on a JSX element.
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

impl LintRule for MouseEventsHaveKeyEvents {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `onMouseOver`/`onMouseOut` have `onFocus`/`onBlur`".to_owned(),
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

        // onMouseOver requires onFocus
        if has_attribute(opening, "onMouseOver", ctx) && !has_attribute(opening, "onFocus", ctx) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "`onMouseOver` must be accompanied by `onFocus` for keyboard accessibility"
                        .to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }

        // onMouseOut requires onBlur
        if has_attribute(opening, "onMouseOut", ctx) && !has_attribute(opening, "onBlur", ctx) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`onMouseOut` must be accompanied by `onBlur` for keyboard accessibility"
                    .to_owned(),
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

    starlint_rule_framework::lint_rule_test!(MouseEventsHaveKeyEvents);

    #[test]
    fn test_flags_mouseover_without_focus() {
        let diags = lint(r"const el = <div onMouseOver={handleOver}>content</div>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_mouseover_with_focus() {
        let diags =
            lint(r"const el = <div onMouseOver={handleOver} onFocus={handleFocus}>content</div>;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_mouseout_without_blur() {
        let diags = lint(r"const el = <div onMouseOut={handleOut}>content</div>;");
        assert_eq!(diags.len(), 1);
    }
}
