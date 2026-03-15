//! Rule: `jsx-a11y/aria-proptypes`
//!
//! Enforce ARIA state and property values are valid.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-proptypes";

/// ARIA attributes that accept only `true` or `false`.
const BOOLEAN_ARIA_PROPS: &[&str] = &[
    "aria-atomic",
    "aria-busy",
    "aria-disabled",
    "aria-grabbed",
    "aria-hidden",
    "aria-modal",
    "aria-multiline",
    "aria-multiselectable",
    "aria-readonly",
    "aria-required",
    "aria-selected",
];

/// ARIA attributes that accept `true`, `false`, or `mixed`.
const TRISTATE_ARIA_PROPS: &[&str] = &["aria-checked", "aria-pressed"];

#[derive(Debug)]
pub struct AriaProptypes;

impl LintRule for AriaProptypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce ARIA state and property values are valid".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        for &attr_id in &*opening.attributes {
            // Extract owned data from ctx.node() before any ctx.report() calls
            let (attr_name, value_id) = {
                let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) else {
                    continue;
                };
                let name = attr.name.clone();
                let val_id = attr.value;
                (name, val_id)
            };

            let name_str = attr_name.as_str();

            if !name_str.starts_with("aria-") {
                continue;
            }

            let Some(value_id) = value_id else {
                continue;
            };

            let val = {
                let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) else {
                    continue;
                };
                lit.value.clone()
            };

            let val_str = val.as_str();

            if BOOLEAN_ARIA_PROPS.contains(&name_str) && val_str != "true" && val_str != "false" {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!("`{name_str}` must be `\"true\"` or `\"false\"`"),
                    span: Span::new(opening.span.start, opening.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }

            if TRISTATE_ARIA_PROPS.contains(&name_str)
                && val_str != "true"
                && val_str != "false"
                && val_str != "mixed"
            {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`{name_str}` must be `\"true\"`, `\"false\"`, or `\"mixed\"`"
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
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(AriaProptypes);

    #[test]
    fn test_flags_invalid_boolean_aria() {
        let diags = lint(r#"const el = <div aria-hidden="yes">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_boolean_aria() {
        let diags = lint(r#"const el = <div aria-hidden="true">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_tristate_mixed() {
        let diags = lint(r#"const el = <div aria-checked="mixed">content</div>;"#);
        assert!(diags.is_empty());
    }
}
