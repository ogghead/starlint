//! Rule: `jsx-a11y/no-redundant-roles`
//!
//! Forbid redundant roles (e.g., `<button role="button">`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-redundant-roles";

/// Mapping of elements to their implicit ARIA roles.
const DEFAULT_ROLE_MAP: &[(&str, &str)] = &[
    ("a", "link"),
    ("article", "article"),
    ("aside", "complementary"),
    ("button", "button"),
    ("footer", "contentinfo"),
    ("form", "form"),
    ("header", "banner"),
    ("img", "img"),
    ("li", "listitem"),
    ("main", "main"),
    ("nav", "navigation"),
    ("ol", "list"),
    ("section", "region"),
    ("table", "table"),
    ("td", "cell"),
    ("textarea", "textbox"),
    ("th", "columnheader"),
    ("tr", "row"),
    ("ul", "list"),
];

#[derive(Debug)]
pub struct NoRedundantRoles;

/// Get the default role for an element.
fn default_role(element: &str) -> Option<&'static str> {
    for &(elem, role) in DEFAULT_ROLE_MAP {
        if elem == element {
            return Some(role);
        }
    }
    None
}

impl LintRule for NoRedundantRoles {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid redundant roles (e.g., `<button role=\"button\">`)".to_owned(),
            category: Category::Style,
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

        // opening.name is a String
        let element_name = opening.name.as_str();

        let Some(implicit_role) = default_role(element_name) else {
            return;
        };

        for attr_id in &*opening.attributes {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() != "role" {
                    continue;
                }

                if let Some(value_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                        let role_val = lit.value.as_str().trim();
                        if role_val == implicit_role {
                            let attr_span = Span::new(attr.span.start, attr.span.end);
                            let fix = FixBuilder::new(
                                "Remove redundant `role` attribute",
                                FixKind::SafeFix,
                            )
                            .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                            .build();
                            ctx.report(Diagnostic {
                                rule_name: RULE_NAME.to_owned(),
                                message: format!(
                                    "`<{element_name}>` has an implicit `role` of `{implicit_role}`. Setting `role=\"{role_val}\"` is redundant"
                                ),
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
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRedundantRoles)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_redundant_button_role() {
        let diags = lint(r#"const el = <button role="button">click</button>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_redundant_nav_role() {
        let diags = lint(r#"const el = <nav role="navigation">menu</nav>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_different_role() {
        let diags = lint(r#"const el = <div role="button">click</div>;"#);
        assert!(diags.is_empty());
    }
}
