//! Rule: `jsx-a11y/anchor-is-valid`
//!
//! Enforce anchors are valid (have href, not `#` or `javascript:`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/anchor-is-valid";

#[derive(Debug)]
pub struct AnchorIsValid;

/// Check if an attribute exists on a JSX opening element.
fn has_attribute(attributes: &[NodeId], name: &str, ctx: &LintContext<'_>) -> bool {
    attributes.iter().any(|&attr_id| {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            attr.name == name
        } else {
            false
        }
    })
}

/// Get string value of an attribute if it's a string literal.
fn get_attr_string_value(
    attributes: &[NodeId],
    attr_name: &str,
    ctx: &LintContext<'_>,
) -> Option<String> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name {
                if let Some(value_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                        return Some(lit.value.clone());
                    }
                }
            }
        }
    }
    None
}

/// Get the span of an attribute's value (including quotes) if it's a string literal.
fn get_attr_value_span(
    attributes: &[NodeId],
    attr_name_str: &str,
    ctx: &LintContext<'_>,
) -> Option<Span> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name_str {
                if let Some(value_id) = attr.value {
                    if let Some(node) = ctx.node(value_id) {
                        let s = node.span();
                        return Some(Span::new(s.start, s.end));
                    }
                }
            }
        }
    }
    None
}

impl LintRule for AnchorIsValid {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce anchors are valid (have href, not `#` or `javascript:`)"
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

        if opening.name != "a" {
            return;
        }

        let opening_span = opening.span;
        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        // Check if href exists
        if !has_attribute(&attrs, "href", ctx) {
            let insert_pos = fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening_span.start, opening_span.end),
            );
            let fix = FixBuilder::new("Add `href` attribute", FixKind::SuggestionFix)
                .insert_at(insert_pos, " href=\"${1:/path}\"")
                .build_snippet();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Anchors must have an `href` attribute".to_owned(),
                span: Span::new(opening_span.start, opening_span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
            return;
        }

        // Check for invalid href values
        if let Some(href) = get_attr_string_value(&attrs, "href", ctx) {
            if href == "#" || href.starts_with("javascript:") {
                let fix = get_attr_value_span(&attrs, "href", ctx).and_then(|val_span| {
                    FixBuilder::new("Replace with a valid URL", FixKind::SuggestionFix)
                        .replace(Span::new(val_span.start, val_span.end), "\"${1:/path}\"")
                        .build_snippet()
                });
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Anchors must have a valid `href` attribute. Avoid `#` or `javascript:` URLs".to_owned(),
                    span: Span::new(opening_span.start, opening_span.end),
                    severity: Severity::Warning,
                    help: Some("Replace with a valid URL".to_owned()),
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

    starlint_rule_framework::lint_rule_test!(AnchorIsValid);

    #[test]
    fn test_flags_anchor_without_href() {
        let diags = lint(r"const el = <a>link</a>;");
        assert_eq!(diags.len(), 1, "should flag anchor without href");
    }

    #[test]
    fn test_flags_anchor_with_hash_href() {
        let diags = lint(r##"const el = <a href="#">link</a>;"##);
        assert_eq!(diags.len(), 1, "should flag anchor with href='#'");
    }

    #[test]
    fn test_allows_anchor_with_valid_href() {
        let diags = lint(r#"const el = <a href="/about">About</a>;"#);
        assert!(diags.is_empty(), "should allow anchor with valid href");
    }
}
