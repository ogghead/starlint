//! Rule: `jsx-a11y/aria-unsupported-elements`
//!
//! Forbid `aria-*` and `role` attributes on elements that don't support them.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-unsupported-elements";

/// Elements that do not support ARIA roles or attributes.
const UNSUPPORTED_ELEMENTS: &[&str] = &[
    "meta", "html", "script", "style", "head", "title", "base", "col",
];

#[derive(Debug)]
pub struct AriaUnsupportedElements;

impl LintRule for AriaUnsupportedElements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Forbid `aria-*` and `role` attributes on elements that don't support them"
                    .to_owned(),
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

        let element_name = opening.name.as_str();

        if !UNSUPPORTED_ELEMENTS.contains(&element_name) {
            return;
        }

        let has_aria_or_role = opening.attributes.iter().any(|&attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
                let name = attr.name.as_str();
                name.starts_with("aria-") || name == "role"
            } else {
                false
            }
        });

        if has_aria_or_role {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`<{element_name}>` does not support ARIA roles or `aria-*` attributes"
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AriaUnsupportedElements)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_aria_on_meta() {
        let diags = lint(r#"const el = <meta aria-hidden="true" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_role_on_script() {
        let diags = lint(r#"const el = <script role="button">content</script>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_aria_on_div() {
        let diags = lint(r#"const el = <div aria-label="hello">content</div>;"#);
        assert!(diags.is_empty());
    }
}
