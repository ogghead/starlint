//! Rule: `jsx-a11y/role-has-required-aria-props`
//!
//! Enforce elements with ARIA roles have required aria-* props.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::jsx_utils::has_jsx_attribute;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/role-has-required-aria-props";

/// Roles and their required ARIA properties.
const ROLE_REQUIRED_PROPS: &[(&str, &[&str])] = &[
    ("checkbox", &["aria-checked"]),
    ("combobox", &["aria-expanded"]),
    ("heading", &["aria-level"]),
    ("meter", &["aria-valuenow"]),
    ("option", &["aria-selected"]),
    ("radio", &["aria-checked"]),
    ("scrollbar", &["aria-controls", "aria-valuenow"]),
    ("separator", &["aria-valuenow"]),
    ("slider", &["aria-valuenow"]),
    ("spinbutton", &["aria-valuenow"]),
    ("switch", &["aria-checked"]),
];

#[derive(Debug)]
pub struct RoleHasRequiredAriaProps;

/// Get the required aria props for a given role.
fn required_props(role: &str) -> Option<&'static [&'static str]> {
    for &(r, props) in ROLE_REQUIRED_PROPS {
        if r == role {
            return Some(props);
        }
    }
    None
}

impl LintRule for RoleHasRequiredAriaProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce elements with ARIA roles have required aria-* props".to_owned(),
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

        // Find the role attribute value
        let mut role_value: Option<String> = None;
        for attr_id in &*opening.attributes {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
                continue;
            };
            if attr.name.as_str() == "role" {
                if let Some(val_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(val_id) {
                        role_value = Some(lit.value.clone());
                    }
                }
                break;
            }
        }

        let Some(role_raw) = role_value else {
            return;
        };

        let role = role_raw.trim();
        let Some(props) = required_props(role) else {
            return;
        };

        let opening_span_start = opening.span.start;
        let opening_span_end = opening.span.end;
        // Collect attribute NodeIds into a vec to avoid borrow conflict
        let attr_ids: Vec<NodeId> = opening.attributes.to_vec();

        for prop in props {
            if !has_jsx_attribute(&attr_ids, prop, ctx) {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Elements with `role=\"{role}\"` must have the `{prop}` attribute"
                    ),
                    span: Span::new(opening_span_start, opening_span_end),
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

    starlint_rule_framework::lint_rule_test!(RoleHasRequiredAriaProps);

    #[test]
    fn test_flags_checkbox_without_aria_checked() {
        let diags = lint(r#"const el = <div role="checkbox">check</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_checkbox_with_aria_checked() {
        let diags = lint(r#"const el = <div role="checkbox" aria-checked="true">check</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_role() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
