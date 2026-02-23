//! Rule: `jsx-a11y/no-aria-hidden-on-focusable`
//!
//! Forbid `aria-hidden="true"` on focusable elements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-aria-hidden-on-focusable";

/// Inherently interactive (focusable) elements.
const INTERACTIVE_ELEMENTS: &[&str] = &["button", "input", "select", "textarea"];

#[derive(Debug)]
pub struct NoAriaHiddenOnFocusable;

/// Check if an attribute with the given name exists on the JSX opening element's attributes.
fn has_attribute(ctx: &LintContext<'_>, attributes: &[NodeId], name: &str) -> bool {
    attributes.iter().any(|attr_id| {
        let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
            return false;
        };
        attr.name.as_str() == name
    })
}

impl LintRule for NoAriaHiddenOnFocusable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `aria-hidden=\"true\"` on focusable elements".to_owned(),
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

        // Check if aria-hidden="true" and capture its attribute span
        let aria_hidden_span = opening.attributes.iter().find_map(|attr_id| {
            let AstNode::JSXAttribute(attr) = ctx.node(*attr_id)? else {
                return None;
            };
            if attr.name.as_str() != "aria-hidden" {
                return None;
            }
            // Check if value is the string "true"
            let val_id = attr.value?;
            let AstNode::StringLiteral(lit) = ctx.node(val_id)? else {
                return None;
            };
            (lit.value.as_str() == "true").then(|| Span::new(attr.span.start, attr.span.end))
        });

        let Some(attr_span) = aria_hidden_span else {
            return;
        };

        let element_name = opening.name.as_str();

        // Check if inherently interactive
        let is_interactive = INTERACTIVE_ELEMENTS.contains(&element_name);

        // <a> with href is focusable
        let is_anchor_with_href =
            element_name == "a" && has_attribute(ctx, &opening.attributes, "href");

        // Any element with tabIndex is focusable
        let has_tabindex = has_attribute(ctx, &opening.attributes, "tabIndex");

        if is_interactive || is_anchor_with_href || has_tabindex {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`aria-hidden=\"true\"` must not be set on focusable elements".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove `aria-hidden` attribute".to_owned(),
                    edits: vec![Edit {
                        span: attr_span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAriaHiddenOnFocusable)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_aria_hidden_on_button() {
        let diags = lint(r#"const el = <button aria-hidden="true">click</button>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_aria_hidden_on_anchor_with_href() {
        let diags = lint(r#"const el = <a href="/about" aria-hidden="true">link</a>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_aria_hidden_on_div() {
        let diags = lint(r#"const el = <div aria-hidden="true">content</div>;"#);
        assert!(diags.is_empty());
    }
}
