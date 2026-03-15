//! Rule: `jsx-a11y/aria-role`
//!
//! Enforce `role` attribute has a valid value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-role";

/// Valid WAI-ARIA roles.
const VALID_ROLES: &[&str] = &[
    "alert",
    "alertdialog",
    "application",
    "article",
    "banner",
    "button",
    "cell",
    "checkbox",
    "columnheader",
    "combobox",
    "complementary",
    "contentinfo",
    "definition",
    "dialog",
    "directory",
    "document",
    "feed",
    "figure",
    "form",
    "grid",
    "gridcell",
    "group",
    "heading",
    "img",
    "link",
    "list",
    "listbox",
    "listitem",
    "log",
    "main",
    "marquee",
    "math",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "navigation",
    "none",
    "note",
    "option",
    "presentation",
    "progressbar",
    "radio",
    "radiogroup",
    "region",
    "row",
    "rowgroup",
    "rowheader",
    "scrollbar",
    "search",
    "searchbox",
    "separator",
    "slider",
    "spinbutton",
    "status",
    "switch",
    "tab",
    "table",
    "tablist",
    "tabpanel",
    "term",
    "textbox",
    "timer",
    "toolbar",
    "tooltip",
    "tree",
    "treegrid",
    "treeitem",
];

#[derive(Debug)]
pub struct AriaRole;

impl LintRule for AriaRole {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `role` attribute has a valid value".to_owned(),
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

        for &attr_id in &*opening.attributes {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) else {
                continue;
            };

            if attr.name.as_str() != "role" {
                continue;
            }

            let Some(value_id) = attr.value else {
                continue;
            };
            if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                let role = lit.value.as_str().trim();
                if !role.is_empty() && !VALID_ROLES.contains(&role) {
                    let attr_span = Span::new(attr.span.start, attr.span.end);
                    let fix = FixBuilder::new(
                        format!("Remove invalid `role=\"{role}\"` attribute"),
                        FixKind::SuggestionFix,
                    )
                    .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                    .build();
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!("`{role}` is not a valid WAI-ARIA role"),
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

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(AriaRole);

    #[test]
    fn test_flags_invalid_role() {
        let diags = lint(r#"const el = <div role="foobar">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_role() {
        let diags = lint(r#"const el = <div role="button">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_role() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
