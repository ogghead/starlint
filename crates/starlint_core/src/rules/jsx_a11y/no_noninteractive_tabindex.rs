//! Rule: `jsx-a11y/no-noninteractive-tabindex`
//!
//! Forbid `tabIndex` on non-interactive elements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-noninteractive-tabindex";

/// Interactive HTML elements that naturally accept tabIndex.
const INTERACTIVE_ELEMENTS: &[&str] = &[
    "a", "button", "input", "select", "textarea", "details", "summary",
];

/// Non-interactive elements (common static elements).
const NON_INTERACTIVE_ELEMENTS: &[&str] = &[
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "span",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

#[derive(Debug)]
pub struct NoNoninteractiveTabindex;

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

/// Get string value and span of an attribute if it's a string literal.
fn get_attr_string_value_and_span(
    attributes: &[NodeId],
    attr_name: &str,
    ctx: &LintContext<'_>,
) -> Option<(String, Span)> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name {
                let attr_span = Span::new(attr.span.start, attr.span.end);
                if let Some(value_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                        return Some((lit.value.clone(), attr_span));
                    }
                }
            }
        }
    }
    None
}

/// Get the span of a named attribute (regardless of value type).
fn get_attr_span(attributes: &[NodeId], attr_name: &str, ctx: &LintContext<'_>) -> Option<Span> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name {
                return Some(Span::new(attr.span.start, attr.span.end));
            }
        }
    }
    None
}

impl LintRule for NoNoninteractiveTabindex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `tabIndex` on non-interactive elements".to_owned(),
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

        let element_name = opening.name.clone();
        let opening_span = opening.span;
        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        // Skip interactive elements
        if INTERACTIVE_ELEMENTS.contains(&element_name.as_str()) {
            return;
        }

        // Only check known non-interactive elements
        if !NON_INTERACTIVE_ELEMENTS.contains(&element_name.as_str()) {
            return;
        }

        // If element has a role, it may be intentionally interactive
        if has_attribute(&attrs, "role", ctx) {
            return;
        }

        // Check for tabIndex attribute
        let tabindex_info = get_attr_string_value_and_span(&attrs, "tabIndex", ctx);
        if let Some((val, attr_span)) = tabindex_info {
            let parsed = val.parse::<i32>().unwrap_or(-1);
            // tabIndex="-1" is acceptable (removes from tab order)
            if parsed >= 0 {
                let fix = FixBuilder::new("Remove `tabIndex` attribute", FixKind::SuggestionFix)
                    .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                    .build();
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`<{element_name}>` is non-interactive and should not have `tabIndex`"
                    ),
                    span: Span::new(opening_span.start, opening_span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        } else if let Some(attr_span) = get_attr_span(&attrs, "tabIndex", ctx) {
            // tabIndex without a value (boolean attribute) defaults to 0
            let fix = FixBuilder::new("Remove `tabIndex` attribute", FixKind::SuggestionFix)
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "`<{element_name}>` is non-interactive and should not have `tabIndex`"
                ),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNoninteractiveTabindex)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_tabindex_on_div() {
        let diags = lint(r#"const el = <div tabIndex="0">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_tabindex_on_button() {
        let diags = lint(r#"const el = <button tabIndex="0">click</button>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_negative_tabindex_on_div() {
        let diags = lint(r#"const el = <div tabIndex="-1">content</div>;"#);
        assert!(diags.is_empty());
    }
}
