//! Shared JSX attribute utilities for lint rules.
//!
//! Provides helpers for inspecting JSX attributes by name, extracting
//! string values, and querying attribute presence. These are used
//! across multiple plugin crates (react, nextjs, etc.).

use starlint_ast::node::AstNode;
use starlint_ast::types::NodeId;

use crate::LintContext;

/// Check if a JSX element has an attribute with the given `name`.
///
/// Iterates over the `attributes` [`NodeId`] slice (from a
/// `JSXOpeningElement`) and returns `true` if any `JSXAttribute` has a
/// matching name.
#[must_use]
pub fn has_jsx_attribute(attributes: &[NodeId], name: &str, ctx: &LintContext<'_>) -> bool {
    attributes.iter().any(|&attr_id| {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            attr.name == name
        } else {
            false
        }
    })
}

/// Get the string value of a JSX attribute by name.
///
/// Returns `Some(value)` if the attribute exists and its value is a
/// [`StringLiteral`](AstNode::StringLiteral). Returns `None` if the
/// attribute is missing, has no value, or has a non-string value.
#[must_use]
pub fn get_jsx_attr_string_value(
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

/// Get the [`NodeId`] of a JSX attribute's value by attribute name.
///
/// Returns `Some(value_id)` if the attribute exists and has a value node.
#[must_use]
pub fn get_jsx_attr_value(
    attributes: &[NodeId],
    attr_name: &str,
    ctx: &LintContext<'_>,
) -> Option<NodeId> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name {
                return attr.value;
            }
        }
    }
    None
}

/// Extract a string value from a JSX attribute value [`NodeId`].
///
/// Returns `Some(value)` if the node is a [`StringLiteral`](AstNode::StringLiteral).
#[must_use]
pub fn get_string_value(ctx: &LintContext<'_>, value: Option<NodeId>) -> Option<String> {
    let id = value?;
    let node = ctx.node(id)?;
    if let AstNode::StringLiteral(lit) = node {
        Some(lit.value.clone())
    } else {
        None
    }
}
