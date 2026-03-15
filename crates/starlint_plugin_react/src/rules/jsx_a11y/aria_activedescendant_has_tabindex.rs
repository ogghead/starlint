//! Rule: `jsx-a11y/aria-activedescendant-has-tabindex`
//!
//! Enforce elements with `aria-activedescendant` are tabbable.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::jsx_utils::{get_jsx_attr_string_value, has_jsx_attribute};
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-activedescendant-has-tabindex";

/// Interactive elements that are naturally tabbable.
const INTERACTIVE_ELEMENTS: &[&str] = &["input", "select", "textarea", "button", "a"];

#[derive(Debug)]
pub struct AriaActivedescendantHasTabindex;

impl LintRule for AriaActivedescendantHasTabindex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce elements with `aria-activedescendant` are tabbable".to_owned(),
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

        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        if !has_jsx_attribute(&attrs, "aria-activedescendant", ctx) {
            return;
        }

        // Check if it is an interactive element
        let is_interactive = INTERACTIVE_ELEMENTS.contains(&opening.name.as_str());

        if is_interactive {
            return;
        }

        // Non-interactive: must have tabIndex
        let has_tabindex = has_jsx_attribute(&attrs, "tabIndex", ctx);
        let tabindex_val = get_jsx_attr_string_value(&attrs, "tabIndex", ctx);
        let is_negative = tabindex_val
            .and_then(|v| v.parse::<i32>().ok())
            .is_some_and(|n| n < 0);

        if !has_tabindex || is_negative {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "An element with `aria-activedescendant` must be tabbable. Add `tabIndex`"
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

    starlint_rule_framework::lint_rule_test!(AriaActivedescendantHasTabindex);

    #[test]
    fn test_flags_div_with_activedescendant_no_tabindex() {
        let diags = lint(r#"const el = <div aria-activedescendant="item-1">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_input_with_activedescendant() {
        let diags = lint(r#"const el = <input aria-activedescendant="item-1" />;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_div_with_activedescendant_and_tabindex() {
        let diags =
            lint(r#"const el = <div aria-activedescendant="item-1" tabIndex="0">content</div>;"#);
        assert!(diags.is_empty());
    }
}
