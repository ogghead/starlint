//! Rule: `jsx-a11y/alt-text`
//!
//! Enforce alt text on `<img>`, `<area>`, `<input type="image">`, and `<object>`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/alt-text";

/// Elements that require alt text.
const ELEMENTS_REQUIRING_ALT: &[&str] = &["img", "area", "object"];

#[derive(Debug)]
pub struct AltText;

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

/// Check if an `<input>` element has `type="image"`.
fn is_input_type_image(attributes: &[NodeId], ctx: &LintContext<'_>) -> bool {
    get_attr_string_value(attributes, "type", ctx).as_deref() == Some("image")
}

impl LintRule for AltText {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Enforce alt text on `<img>`, `<area>`, `<input type=\"image\">`, and `<object>`"
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

        let name = opening.name.as_str();
        let opening_span = opening.span;
        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        let needs_alt = ELEMENTS_REQUIRING_ALT.contains(&name)
            || (name == "input" && is_input_type_image(&attrs, ctx));

        if !needs_alt {
            return;
        }

        let has_alt = has_attribute(&attrs, "alt", ctx);

        // For <object>, also accept aria-label or aria-labelledby
        let has_aria_label = has_attribute(&attrs, "aria-label", ctx)
            || has_attribute(&attrs, "aria-labelledby", ctx);

        if name == "object" {
            if !has_alt && !has_aria_label {
                let insert_pos = starlint_rule_framework::fix_utils::jsx_attr_insert_offset(
                    ctx.source_text(),
                    Span::new(opening_span.start, opening_span.end),
                );
                let fix = FixBuilder::new("Add `aria-label` attribute", FixKind::SuggestionFix)
                    .insert_at(insert_pos, " aria-label=\"${1:object description}\"")
                    .build_snippet();

                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "`<object>` elements must have an `alt`, `aria-label`, or `aria-labelledby` attribute".to_owned(),
                    span: Span::new(opening_span.start, opening_span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        } else if !has_alt {
            let insert_pos = starlint_rule_framework::fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening_span.start, opening_span.end),
            );
            let fix = FixBuilder::new("Add `alt` attribute", FixKind::SuggestionFix)
                .insert_at(insert_pos, " alt=\"${1:descriptive text}\"")
                .build_snippet();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`<{name}>` elements must have an `alt` attribute"),
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

    starlint_rule_framework::lint_rule_test!(AltText);

    #[test]
    fn test_flags_img_without_alt() {
        let diags = lint(r#"const el = <img src="foo.png" />;"#);
        assert_eq!(diags.len(), 1, "should flag img without alt");
    }

    #[test]
    fn test_allows_img_with_alt() {
        let diags = lint(r#"const el = <img src="foo.png" alt="A photo" />;"#);
        assert!(diags.is_empty(), "should allow img with alt");
    }

    #[test]
    fn test_flags_input_type_image_without_alt() {
        let diags = lint(r#"const el = <input type="image" src="submit.png" />;"#);
        assert_eq!(diags.len(), 1, "should flag input type=image without alt");
    }
}
