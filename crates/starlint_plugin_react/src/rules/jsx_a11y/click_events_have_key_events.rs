//! Rule: `jsx-a11y/click-events-have-key-events`
//!
//! Enforce `onClick` is accompanied by `onKeyDown`, `onKeyUp`, or `onKeyPress`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::jsx_utils::has_jsx_attribute;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/click-events-have-key-events";

#[derive(Debug)]
pub struct ClickEventsHaveKeyEvents;

impl LintRule for ClickEventsHaveKeyEvents {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `onClick` is accompanied by a keyboard event handler".to_owned(),
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

        if !has_jsx_attribute(&opening.attributes, "onClick", ctx) {
            return;
        }

        let has_key_event = has_jsx_attribute(&opening.attributes, "onKeyDown", ctx)
            || has_jsx_attribute(&opening.attributes, "onKeyUp", ctx)
            || has_jsx_attribute(&opening.attributes, "onKeyPress", ctx);

        if !has_key_event {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Elements with `onClick` must have a keyboard event handler (`onKeyDown`, `onKeyUp`, or `onKeyPress`)".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(ClickEventsHaveKeyEvents);

    #[test]
    fn test_flags_onclick_without_key_event() {
        let diags = lint(r"const el = <div onClick={handleClick}>content</div>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_onclick_with_onkeydown() {
        let diags =
            lint(r"const el = <div onClick={handleClick} onKeyDown={handleKey}>content</div>;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_onclick() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
