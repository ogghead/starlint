//! Rule: `jsx-a11y/aria-props`
//!
//! Enforce valid `aria-*` attribute names.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-props";

/// All valid WAI-ARIA 1.1 attribute names.
const VALID_ARIA_PROPS: &[&str] = &[
    "aria-activedescendant",
    "aria-atomic",
    "aria-autocomplete",
    "aria-busy",
    "aria-checked",
    "aria-colcount",
    "aria-colindex",
    "aria-colspan",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-expanded",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-level",
    "aria-live",
    "aria-modal",
    "aria-multiline",
    "aria-multiselectable",
    "aria-orientation",
    "aria-owns",
    "aria-placeholder",
    "aria-posinset",
    "aria-pressed",
    "aria-readonly",
    "aria-relevant",
    "aria-required",
    "aria-roledescription",
    "aria-rowcount",
    "aria-rowindex",
    "aria-rowspan",
    "aria-selected",
    "aria-setsize",
    "aria-sort",
    "aria-valuemax",
    "aria-valuemin",
    "aria-valuenow",
    "aria-valuetext",
];

#[derive(Debug)]
pub struct AriaProps;

impl LintRule for AriaProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce valid `aria-*` attribute names".to_owned(),
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

        for attr_id in &opening.attributes {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
                continue;
            };

            let name_str = attr.name.as_str();
            // Skip namespaced names (contain ':')
            if name_str.contains(':') {
                continue;
            }

            if name_str.starts_with("aria-") && !VALID_ARIA_PROPS.contains(&name_str) {
                let attr_span = Span::new(attr.span.start, attr.span.end);
                let fix = FixBuilder::new(
                    format!("Remove invalid `{name_str}` attribute"),
                    FixKind::SafeFix,
                )
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                .build();
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!("`{name_str}` is not a valid WAI-ARIA attribute"),
                    span: Span::new(opening.span.start, opening.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AriaProps)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_invalid_aria_prop() {
        let diags = lint(r#"const el = <div aria-foobar="true">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_aria_prop() {
        let diags = lint(r#"const el = <div aria-label="hello">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_ignores_non_aria_attributes() {
        let diags = lint(r#"const el = <div data-custom="true">content</div>;"#);
        assert!(diags.is_empty());
    }
}
