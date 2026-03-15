//! Rule: `jsx-a11y/anchor-has-content`
//!
//! Enforce anchors have content.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::jsx_utils::has_jsx_attribute;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/anchor-has-content";

#[derive(Debug)]
pub struct AnchorHasContent;

impl LintRule for AnchorHasContent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce anchors have content".to_owned(),
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

        if opening.name != "a" {
            return;
        }

        // If the element has children, it has content
        if !element.children.is_empty() {
            return;
        }

        let opening_span = opening.span;
        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        // Check for aria-label or aria-labelledby as alternative content
        let has_accessible_content = has_jsx_attribute(&attrs, "aria-label", ctx)
            || has_jsx_attribute(&attrs, "aria-labelledby", ctx);

        if !has_accessible_content {
            let insert_pos = fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening_span.start, opening_span.end),
            );
            let fix = FixBuilder::new("Add `aria-label` attribute", FixKind::SuggestionFix)
                .insert_at(insert_pos, " aria-label=\"${1:link text}\"")
                .build_snippet();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Anchors must have content. Provide child text, `aria-label`, or `aria-labelledby`".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(AnchorHasContent);

    #[test]
    fn test_flags_self_closing_anchor() {
        let diags = lint(r#"const el = <a href="/about" />;"#);
        assert_eq!(
            diags.len(),
            1,
            "should flag self-closing anchor without content"
        );
    }

    #[test]
    fn test_allows_anchor_with_children() {
        let diags = lint(r#"const el = <a href="/about">About</a>;"#);
        assert!(diags.is_empty(), "should allow anchor with text children");
    }

    #[test]
    fn test_allows_self_closing_anchor_with_aria_label() {
        let diags = lint(r#"const el = <a href="/about" aria-label="About page" />;"#);
        assert!(
            diags.is_empty(),
            "should allow self-closing anchor with aria-label"
        );
    }
}
