//! Rule: `jsx-a11y/role-supports-aria-props`
//!
//! Enforce aria-* props are supported by the element's role.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/role-supports-aria-props";

/// Roles and aria props that they explicitly do NOT support.
/// The `presentation` and `none` roles should have no aria-* props at all.
const ROLES_WITHOUT_ARIA: &[&str] = &["presentation", "none"];

/// Global ARIA props supported by all roles.
const GLOBAL_ARIA_PROPS: &[&str] = &[
    "aria-atomic",
    "aria-busy",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-live",
    "aria-owns",
    "aria-relevant",
    "aria-roledescription",
];

#[derive(Debug)]
pub struct RoleSupportAriaProps;

/// Get the string value of a JSX attribute's value (if it's a string literal).
fn get_attr_string_value(
    attr: &starlint_ast::node::JSXAttributeNode,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let value_id = attr.value?;
    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
        Some(lit.value.clone())
    } else {
        None
    }
}

impl LintRule for RoleSupportAriaProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce aria-* props are supported by the element's role".to_owned(),
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
        for attr_id in &opening.attributes {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() == "role" {
                    role_value = get_attr_string_value(attr, ctx);
                    break;
                }
            }
        }

        let Some(role_raw) = role_value else {
            return;
        };

        let role = role_raw.trim();

        // presentation/none roles should not have aria-* props (except global ones in some specs)
        if !ROLES_WITHOUT_ARIA.contains(&role) {
            return;
        }

        // Collect aria attribute names and report (avoid borrow conflict with ctx)
        let violations: Vec<(String, Span)> = opening
            .attributes
            .iter()
            .filter_map(|attr_id| {
                if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                    let name_str = attr.name.as_str();
                    if name_str.starts_with("aria-") && !GLOBAL_ARIA_PROPS.contains(&name_str) {
                        return Some((
                            name_str.to_owned(),
                            Span::new(opening.span.start, opening.span.end),
                        ));
                    }
                }
                None
            })
            .collect();

        for (name_str, span) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`{name_str}` is not supported by `role=\"{role}\"`"),
                span,
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

    starlint_rule_framework::lint_rule_test!(RoleSupportAriaProps);

    #[test]
    fn test_flags_aria_checked_on_presentation() {
        let diags =
            lint(r#"const el = <div role="presentation" aria-checked="true">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_aria_label_on_presentation() {
        let diags =
            lint(r#"const el = <div role="presentation" aria-label="hello">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_aria_on_button_role() {
        let diags = lint(r#"const el = <div role="button" aria-pressed="true">click</div>;"#);
        assert!(diags.is_empty());
    }
}
