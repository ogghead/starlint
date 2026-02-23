//! Rule: `jsx-a11y/heading-has-content`
//!
//! Enforce heading elements (`h1`-`h6`) have content.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/heading-has-content";

/// Heading element names.
const HEADINGS: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6"];

#[derive(Debug)]
pub struct HeadingHasContent;

/// Check if an attribute exists on a JSX opening element by examining its attribute `NodeIds`.
fn has_attribute(attributes: &[NodeId], name: &str, ctx: &LintContext<'_>) -> bool {
    attributes.iter().any(|&attr_id| {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            attr.name == name
        } else {
            false
        }
    })
}

impl LintRule for HeadingHasContent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce heading elements (`h1`-`h6`) have content".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };

        let element_name = opening.name.clone();

        if !HEADINGS.contains(&element_name.as_str()) {
            return;
        }

        // If the element has children, it has content
        if !element.children.is_empty() {
            return;
        }

        let opening_span = opening.span;
        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        // Check for aria-label or aria-labelledby as alternative content
        let has_accessible_content = has_attribute(&attrs, "aria-label", ctx)
            || has_attribute(&attrs, "aria-labelledby", ctx);

        if !has_accessible_content {
            let insert_pos = fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening_span.start, opening_span.end),
            );
            let fix = FixBuilder::new("Add `aria-label` attribute", FixKind::SuggestionFix)
                .insert_at(insert_pos, " aria-label=\"${1:heading text}\"")
                .build_snippet();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`<{element_name}>` must have content. Provide child text, `aria-label`, or `aria-labelledby`"),
                span: Span::new(opening_span.start, opening_span.end),
                severity: Severity::Warning,
                help: None,
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(HeadingHasContent)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_heading() {
        let diags = lint(r"const el = <h1 />;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_heading_with_children() {
        let diags = lint(r"const el = <h1>Title</h1>;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_heading_with_aria_label() {
        let diags = lint(r#"const el = <h1 aria-label="Title" />;"#);
        assert!(diags.is_empty());
    }
}
