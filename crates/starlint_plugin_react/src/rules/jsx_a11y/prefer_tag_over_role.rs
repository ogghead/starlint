//! Rule: `jsx-a11y/prefer-tag-over-role`
//!
//! Prefer using semantic HTML tags over ARIA roles.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/prefer-tag-over-role";

/// Mapping of ARIA roles to preferred semantic HTML tags.
const ROLE_TO_TAG: &[(&str, &str)] = &[
    ("banner", "<header>"),
    ("button", "<button>"),
    ("cell", "<td>"),
    ("columnheader", "<th>"),
    ("complementary", "<aside>"),
    ("contentinfo", "<footer>"),
    ("form", "<form>"),
    ("heading", "<h1>-<h6>"),
    ("img", "<img>"),
    ("link", "<a>"),
    ("list", "<ul> or <ol>"),
    ("listitem", "<li>"),
    ("main", "<main>"),
    ("navigation", "<nav>"),
    ("row", "<tr>"),
    ("table", "<table>"),
];

#[derive(Debug)]
pub struct PreferTagOverRole;

/// Get the preferred tag for a given role.
fn preferred_tag(role: &str) -> Option<&'static str> {
    for &(r, tag) in ROLE_TO_TAG {
        if r == role {
            return Some(tag);
        }
    }
    None
}

impl LintRule for PreferTagOverRole {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer using semantic HTML tags over ARIA roles".to_owned(),
            category: Category::Suggestion,
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
                if let Some(tag) = preferred_tag(role) {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "Prefer using the `{tag}` element instead of `role=\"{role}\"`"
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferTagOverRole)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_role_button_on_div() {
        let diags = lint(r#"const el = <div role="button">click</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_role_navigation() {
        let diags = lint(r#"const el = <div role="navigation">menu</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_custom_role() {
        let diags = lint(r#"const el = <div role="dialog">content</div>;"#);
        assert!(diags.is_empty());
    }
}
