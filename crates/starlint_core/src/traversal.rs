//! Single-pass AST traversal with interest-based rule dispatch.
//!
//! Traverses the [`AstTree`] once, dispatching each node **only** to rules that
//! declared interest in that [`AstNodeType`] via [`LintRule::run_on_types`].
//! Rules that return `None` (the default) receive all nodes.

use std::path::Path;

use oxc_semantic::Semantic;

use starlint_ast::node_type::{AST_NODE_TYPE_COUNT, AstNodeType};
use starlint_ast::tree::AstTree;
use starlint_ast::types::NodeId;

use crate::lint_rule::{LintContext, LintRule};
use starlint_plugin_sdk::diagnostic::Diagnostic;

/// Maps [`AstNodeType`] discriminants to the rule indices that handle them.
///
/// Built once from rule interest declarations. Rules that return `None` from
/// [`LintRule::run_on_types`] go into the wildcard list and receive every node.
pub struct LintDispatchTable {
    /// Per-AstNodeType rule indices for enter. Index = `AstNodeType as usize`.
    enter: Vec<Vec<usize>>,
    /// Per-AstNodeType rule indices for leave.
    leave: Vec<Vec<usize>>,
    /// Rules that receive ALL nodes on enter (wildcard).
    enter_all: Vec<usize>,
    /// Rules that receive ALL nodes on leave.
    leave_all: Vec<usize>,
}

impl LintDispatchTable {
    /// Build the dispatch table from a set of rules with their original indices.
    pub fn build_from_indices(rules: &[Box<dyn LintRule>], traversal_indices: &[usize]) -> Self {
        let mut table = Self {
            enter: vec![Vec::new(); AST_NODE_TYPE_COUNT],
            leave: vec![Vec::new(); AST_NODE_TYPE_COUNT],
            enter_all: Vec::new(),
            leave_all: Vec::new(),
        };

        for &idx in traversal_indices {
            let Some(rule) = rules.get(idx) else {
                continue;
            };

            match rule.run_on_types() {
                Some(types) => {
                    for &ty in types {
                        if let Some(entry) = table.enter.get_mut(ty.index()) {
                            entry.push(idx);
                        }
                    }
                }
                None => table.enter_all.push(idx),
            }

            match rule.leave_on_types() {
                Some(types) => {
                    for &ty in types {
                        if let Some(entry) = table.leave.get_mut(ty.index()) {
                            entry.push(idx);
                        }
                    }
                }
                None => table.leave_all.push(idx),
            }
        }

        table
    }
}

/// Traverse an [`AstTree`] and dispatch to [`LintRule`]s using a pre-built table.
///
/// Returns diagnostics from all rules combined.
pub fn traverse_ast_tree(
    tree: &AstTree,
    rules: &[Box<dyn LintRule>],
    table: &LintDispatchTable,
    run_once_indices: &[usize],
    source_text: &str,
    file_path: &Path,
    semantic: Option<&Semantic<'_>>,
) -> Vec<Diagnostic> {
    // Per-file active-rule bitmask.
    let active: Vec<bool> = rules
        .iter()
        .map(|r| r.should_run_on_file(source_text, file_path))
        .collect();

    let mut all_diagnostics = Vec::new();

    // Build per-node dispatch lists (resolving indices + active mask).
    let has_traversal = active.iter().any(|&a| a);
    if has_traversal {
        let mut ctx = match semantic {
            Some(sem) => LintContext::with_semantic(tree, source_text, file_path, sem),
            None => LintContext::new(tree, source_text, file_path),
        };

        // Walk the tree in pre-order (same order as nodes Vec).
        walk_and_dispatch(tree, rules, table, &active, &mut ctx);

        all_diagnostics.extend(ctx.into_diagnostics());
    }

    // Run run_once for rules that only need file-level checks.
    if !run_once_indices.is_empty() {
        let mut ctx = match semantic {
            Some(sem) => LintContext::with_semantic(tree, source_text, file_path, sem),
            None => LintContext::new(tree, source_text, file_path),
        };
        for &idx in run_once_indices {
            if active.get(idx).copied().unwrap_or(false) {
                if let Some(rule) = rules.get(idx) {
                    rule.run_once(&mut ctx);
                }
            }
        }
        all_diagnostics.extend(ctx.into_diagnostics());
    }

    all_diagnostics
}

/// Walk the `AstTree` in pre-order and dispatch enter/leave to rules.
fn walk_and_dispatch(
    tree: &AstTree,
    rules: &[Box<dyn LintRule>],
    table: &LintDispatchTable,
    active: &[bool],
    ctx: &mut LintContext<'_>,
) {
    walk_node_recursive(tree, NodeId::ROOT, rules, table, active, ctx);
}

/// Recursively walk a node and its children, dispatching enter/leave.
fn walk_node_recursive(
    tree: &AstTree,
    node_id: NodeId,
    rules: &[Box<dyn LintRule>],
    table: &LintDispatchTable,
    active: &[bool],
    ctx: &mut LintContext<'_>,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };
    let node_type = AstNodeType::from(node);
    let ty_index = node_type.index();

    // Enter dispatch.
    if let Some(targeted) = table.enter.get(ty_index) {
        for &idx in targeted {
            if active.get(idx).copied().unwrap_or(false) {
                if let Some(rule) = rules.get(idx) {
                    rule.run(node_id, node, ctx);
                }
            }
        }
    }
    for &idx in &table.enter_all {
        if active.get(idx).copied().unwrap_or(false) {
            if let Some(rule) = rules.get(idx) {
                rule.run(node_id, node, ctx);
            }
        }
    }

    // Visit children.
    for &child_id in tree.children(node_id) {
        walk_node_recursive(tree, child_id, rules, table, active, ctx);
    }

    // Leave dispatch.
    if let Some(targeted) = table.leave.get(ty_index) {
        for &idx in targeted {
            if active.get(idx).copied().unwrap_or(false) {
                if let Some(rule) = rules.get(idx) {
                    rule.leave(node_id, node, ctx);
                }
            }
        }
    }
    for &idx in &table.leave_all {
        if active.get(idx).copied().unwrap_or(false) {
            if let Some(rule) = rules.get(idx) {
                rule.leave(node_id, node, ctx);
            }
        }
    }
}
