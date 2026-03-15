//! Shared AST utilities for lint rules.
//!
//! Provides helpers for extracting information from AST nodes,
//! such as property key names and literal detection.

use starlint_ast::node::AstNode;
use starlint_ast::types::NodeId;

use crate::LintContext;

/// Extract a static key name from a property/method key [`NodeId`].
///
/// Handles the common cases: identifier names, string literals, and
/// numeric literals. Returns `None` for computed keys or unsupported
/// node types.
///
/// # Supported node types
///
/// - [`IdentifierReference`](AstNode::IdentifierReference) → `ident.name`
/// - [`BindingIdentifier`](AstNode::BindingIdentifier) → `ident.name`
/// - [`StringLiteral`](AstNode::StringLiteral) → `lit.value`
/// - [`NumericLiteral`](AstNode::NumericLiteral) → `lit.raw`
#[must_use]
pub fn extract_static_key_name(key_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(key_id)? {
        AstNode::IdentifierReference(ident) => Some(ident.name.clone()),
        AstNode::BindingIdentifier(ident) => Some(ident.name.clone()),
        AstNode::StringLiteral(lit) => Some(lit.value.clone()),
        AstNode::NumericLiteral(lit) => Some(lit.raw.clone()),
        _ => None,
    }
}

/// Check if an expression is an `expect(...)` call or a chain like
/// `expect(...).not`.
///
/// Recursively walks member expression chains (e.g., `expect(x).not.to`)
/// to find the root `expect()` call.
#[must_use]
pub fn is_expect_chain(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::CallExpression(call)) => {
            matches!(ctx.node(call.callee), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect")
        }
        Some(AstNode::StaticMemberExpression(member)) => is_expect_chain(member.object, ctx),
        _ => false,
    }
}

/// Walk up the AST parent chain to check if `node_id` is inside a
/// callback of a function matching one of `names`.
///
/// Returns `true` if an ancestor `CallExpression` has a callee that is an
/// `IdentifierReference` whose name is in `names`.
#[must_use]
pub fn is_inside_call_with_names(node_id: NodeId, ctx: &LintContext<'_>, names: &[&str]) -> bool {
    let tree = ctx.tree();
    let mut current = tree.parent(node_id);
    while let Some(pid) = current {
        if let Some(AstNode::CallExpression(call)) = tree.get(pid) {
            if let Some(AstNode::IdentifierReference(id)) = tree.get(call.callee) {
                if names.contains(&id.name.as_str()) {
                    return true;
                }
            }
        }
        current = tree.parent(pid);
    }
    false
}
