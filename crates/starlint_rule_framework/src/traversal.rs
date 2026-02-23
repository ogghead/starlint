//! Single-pass AST traversal with interest-based rule dispatch.
//!
//! Traverses the [`AstTree`] once, dispatching each node **only** to rules that
//! declared interest in that [`AstNodeType`] via [`LintRule::run_on_types`].
//! Rules that return `None` (the default) receive all nodes.

use std::path::Path;

use smallvec::SmallVec;
use starlint_ast::node_type::{AST_NODE_TYPE_COUNT, AstNodeType};
use starlint_ast::tree::AstTree;
use starlint_ast::types::NodeId;
use starlint_scope::ScopeData;

use crate::lint_rule::{LintContext, LintRule};
use starlint_plugin_sdk::diagnostic::Diagnostic;

/// Inline capacity for per-node-type rule index lists.
///
/// Most node types have 0–3 interested rules; 4 covers the common case
/// without heap allocation.
const DISPATCH_INLINE_CAP: usize = 4;

/// A small list of rule indices — inlined for up to [`DISPATCH_INLINE_CAP`] entries.
type RuleIndices = SmallVec<[usize; DISPATCH_INLINE_CAP]>;

/// Maps [`AstNodeType`] discriminants to the rule indices that handle them.
///
/// Built once from rule interest declarations. Rules that return `None` from
/// [`LintRule::run_on_types`] go into the wildcard list and receive every node.
pub struct LintDispatchTable {
    /// Per-AstNodeType rule indices for enter. Index = `AstNodeType as usize`.
    enter: Vec<RuleIndices>,
    /// Per-AstNodeType rule indices for leave.
    leave: Vec<RuleIndices>,
    /// Rules that receive ALL nodes on enter (wildcard).
    enter_all: RuleIndices,
    /// Rules that receive ALL nodes on leave.
    leave_all: RuleIndices,
}

impl LintDispatchTable {
    /// Build the dispatch table from a set of rules with their original indices.
    pub fn build_from_indices(rules: &[Box<dyn LintRule>], traversal_indices: &[usize]) -> Self {
        let mut table = Self {
            enter: (0..AST_NODE_TYPE_COUNT).map(|_| SmallVec::new()).collect(),
            leave: (0..AST_NODE_TYPE_COUNT).map(|_| SmallVec::new()).collect(),
            enter_all: SmallVec::new(),
            leave_all: SmallVec::new(),
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

    /// Build a per-file filtered dispatch table containing only active rules.
    ///
    /// Eliminates per-node active-checking overhead in the traversal hot path
    /// by pre-filtering rule indices against the active mask once per file.
    fn filtered(&self, active: &[bool]) -> Self {
        let is_active = |idx: &usize| active.get(*idx).copied().unwrap_or(false);

        Self {
            enter: self
                .enter
                .iter()
                .map(|v| v.iter().copied().filter(is_active).collect())
                .collect(),
            leave: self
                .leave
                .iter()
                .map(|v| v.iter().copied().filter(is_active).collect())
                .collect(),
            enter_all: self.enter_all.iter().copied().filter(is_active).collect(),
            leave_all: self.leave_all.iter().copied().filter(is_active).collect(),
        }
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
    scope_data: Option<&ScopeData>,
) -> Vec<Diagnostic> {
    // Per-file active-rule mask.
    let active: Vec<bool> = rules
        .iter()
        .map(|r| r.should_run_on_file(source_text, file_path))
        .collect();

    let mut all_diagnostics = Vec::new();

    // Build a per-file filtered dispatch table containing only active rules.
    // This eliminates per-node active-checking in the traversal hot path.
    let filtered = table.filtered(&active);
    let has_traversal = !filtered.enter_all.is_empty()
        || !filtered.leave_all.is_empty()
        || filtered.enter.iter().any(|v| !v.is_empty())
        || filtered.leave.iter().any(|v| !v.is_empty());

    if has_traversal {
        let mut ctx = match scope_data {
            Some(sd) => LintContext::with_scope_data(tree, source_text, file_path, sd),
            None => LintContext::new(tree, source_text, file_path),
        };

        // Walk the tree in pre-order — the filtered table has no inactive rules.
        walk_and_dispatch(tree, rules, &filtered, &mut ctx);

        all_diagnostics.extend(ctx.into_diagnostics());
    }

    // Run run_once for active rules that only need file-level checks.
    if !run_once_indices.is_empty() {
        let mut ctx = match scope_data {
            Some(sd) => LintContext::with_scope_data(tree, source_text, file_path, sd),
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
    ctx: &mut LintContext<'_>,
) {
    walk_node_recursive(tree, NodeId::ROOT, rules, table, ctx);
}

/// Recursively walk a node and its children, dispatching enter/leave.
///
/// The dispatch table is pre-filtered to contain only active rules,
/// so no per-node active-checking is needed in this hot path.
fn walk_node_recursive(
    tree: &AstTree,
    node_id: NodeId,
    rules: &[Box<dyn LintRule>],
    table: &LintDispatchTable,
    ctx: &mut LintContext<'_>,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };
    let node_type = AstNodeType::from(node);
    let ty_index = node_type.index();

    // Enter dispatch — table is pre-filtered, no active check needed.
    if let Some(targeted) = table.enter.get(ty_index) {
        for &idx in targeted {
            if let Some(rule) = rules.get(idx) {
                rule.run(node_id, node, ctx);
            }
        }
    }
    for &idx in &table.enter_all {
        if let Some(rule) = rules.get(idx) {
            rule.run(node_id, node, ctx);
        }
    }

    // Visit children.
    for &child_id in tree.children(node_id) {
        walk_node_recursive(tree, child_id, rules, table, ctx);
    }

    // Leave dispatch — table is pre-filtered, no active check needed.
    if let Some(targeted) = table.leave.get(ty_index) {
        for &idx in targeted {
            if let Some(rule) = rules.get(idx) {
                rule.leave(node_id, node, ctx);
            }
        }
    }
    for &idx in &table.leave_all {
        if let Some(rule) = rules.get(idx) {
            rule.leave(node_id, node, ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use starlint_ast::node::AstNode;
    use starlint_ast::node_type::AstNodeType;
    use starlint_ast::tree::AstTree;
    use starlint_ast::types::NodeId;
    use starlint_plugin_sdk::diagnostic::{Severity, Span};
    use starlint_plugin_sdk::rule::{Category, RuleMeta};
    use std::path::Path;

    use super::*;
    use crate::lint_rule::{LintContext, LintRule};

    // ── Mock rules for testing dispatch ──

    /// Rule that records enter calls via a diagnostic.
    #[derive(Debug)]
    struct EnterCountRule {
        name: &'static str,
    }

    impl LintRule for EnterCountRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: self.name.to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run(&self, _node_id: NodeId, _node: &AstNode, ctx: &mut LintContext<'_>) {
            ctx.report_warning(self.name, "entered", Span::new(0, 0));
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            None // wildcard — receives all nodes
        }
    }

    /// Rule that records leave calls via a diagnostic.
    #[derive(Debug)]
    struct LeaveCountRule;

    impl LintRule for LeaveCountRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "leave-rule".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run(&self, _node_id: NodeId, _node: &AstNode, _ctx: &mut LintContext<'_>) {}

        fn leave(&self, _node_id: NodeId, _node: &AstNode, ctx: &mut LintContext<'_>) {
            ctx.report_warning("leave-rule", "left", Span::new(0, 0));
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            None
        }

        fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
            None // wildcard leave
        }
    }

    /// Rule that targets specific node types for leave.
    #[derive(Debug)]
    struct TargetedLeaveRule;

    impl LintRule for TargetedLeaveRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "targeted-leave".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn leave(&self, _node_id: NodeId, _node: &AstNode, ctx: &mut LintContext<'_>) {
            ctx.report_warning("targeted-leave", "targeted-left", Span::new(0, 0));
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::ExpressionStatement])
        }

        fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
            Some(&[AstNodeType::ExpressionStatement])
        }
    }

    /// Rule that only uses `run_once` (no traversal).
    #[derive(Debug)]
    struct RunOnceRule;

    impl LintRule for RunOnceRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "run-once".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn needs_traversal(&self) -> bool {
            false
        }

        fn run_once(&self, ctx: &mut LintContext<'_>) {
            ctx.report_warning("run-once", "ran once", Span::new(0, 0));
        }
    }

    /// Rule that always skips files.
    #[derive(Debug)]
    struct SkipFileRule;

    impl LintRule for SkipFileRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "skip-file".to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            }
        }

        fn run(&self, _node_id: NodeId, _node: &AstNode, ctx: &mut LintContext<'_>) {
            ctx.report_warning("skip-file", "should not appear", Span::new(0, 0));
        }

        fn should_run_on_file(&self, _source_text: &str, _file_path: &Path) -> bool {
            false
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            None
        }
    }

    /// Helper: parse source and run traversal with given rules.
    fn run_traversal(source: &str, rules: &[Box<dyn LintRule>]) -> Vec<Diagnostic> {
        let path = Path::new("test.js");
        let options = starlint_parser::ParseOptions::from_path(path);
        let tree = starlint_parser::parse(source, options).tree;

        let traversal_indices: Vec<usize> = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.needs_traversal())
            .map(|(i, _)| i)
            .collect();
        let run_once_indices: Vec<usize> = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.needs_traversal())
            .map(|(i, _)| i)
            .collect();
        let table = LintDispatchTable::build_from_indices(rules, &traversal_indices);
        traverse_ast_tree(&tree, rules, &table, &run_once_indices, source, path, None)
    }

    #[test]
    fn test_wildcard_enter_dispatches_to_all_nodes() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(EnterCountRule { name: "wildcard" })];
        let diags = run_traversal("1;", &rules);
        assert!(
            !diags.is_empty(),
            "wildcard rule should produce diagnostics for each node"
        );
        assert!(
            diags.iter().all(|d| d.rule_name == "wildcard"),
            "all diagnostics should be from the wildcard rule"
        );
    }

    #[test]
    fn test_wildcard_leave_dispatches() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(LeaveCountRule)];
        let diags = run_traversal("1;", &rules);
        assert!(
            !diags.is_empty(),
            "wildcard leave rule should produce diagnostics"
        );
        assert!(
            diags.iter().all(|d| d.message == "left"),
            "all should be leave diagnostics"
        );
    }

    #[test]
    fn test_targeted_leave_dispatches() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(TargetedLeaveRule)];
        let diags = run_traversal("1;", &rules);
        // The source "1;" should parse as an ExpressionStatement, triggering the targeted leave.
        assert!(
            diags
                .iter()
                .any(|d| d.rule_name == "targeted-leave" && d.message == "targeted-left"),
            "targeted leave should fire on ExpressionStatement"
        );
    }

    #[test]
    fn test_run_once_fires() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RunOnceRule)];
        let diags = run_traversal("const x = 1;", &rules);
        assert_eq!(
            diags.len(),
            1,
            "run_once should produce exactly one diagnostic"
        );
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("run-once"),
            "should be from the run-once rule"
        );
    }

    #[test]
    fn test_should_run_on_file_filters_rule() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(SkipFileRule)];
        let diags = run_traversal("debugger;", &rules);
        assert!(
            diags.is_empty(),
            "rule that returns false from should_run_on_file should produce no diagnostics"
        );
    }

    #[test]
    fn test_mixed_traversal_and_run_once() {
        let rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(EnterCountRule { name: "enter" }),
            Box::new(RunOnceRule),
        ];
        let diags = run_traversal("1;", &rules);
        let enter_count = diags.iter().filter(|d| d.rule_name == "enter").count();
        let once_count = diags.iter().filter(|d| d.rule_name == "run-once").count();
        assert!(enter_count > 0, "enter rule should fire");
        assert_eq!(once_count, 1, "run-once should fire exactly once");
    }

    #[test]
    fn test_empty_rules_produces_no_diagnostics() {
        let rules: Vec<Box<dyn LintRule>> = vec![];
        let diags = run_traversal("const x = 1;", &rules);
        assert!(diags.is_empty(), "no rules should produce no diagnostics");
    }

    #[test]
    fn test_empty_tree_with_wildcard_rule() {
        let tree = AstTree::new();
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(EnterCountRule { name: "wildcard" })];
        let table = LintDispatchTable::build_from_indices(&rules, &[0]);
        let diags = traverse_ast_tree(&tree, &rules, &table, &[], "", Path::new("test.js"), None);
        assert!(
            diags.is_empty(),
            "empty tree should produce no diagnostics even with wildcard rule"
        );
    }

    #[test]
    fn test_run_once_with_should_run_on_file_false() {
        /// Run-once rule that skips all files.
        #[derive(Debug)]
        struct SkipOnceRule;

        impl LintRule for SkipOnceRule {
            fn meta(&self) -> RuleMeta {
                RuleMeta {
                    name: "skip-once".to_owned(),
                    description: String::new(),
                    category: Category::Correctness,
                    default_severity: Severity::Warning,
                }
            }

            fn needs_traversal(&self) -> bool {
                false
            }

            fn run_once(&self, ctx: &mut LintContext<'_>) {
                ctx.report_warning("skip-once", "should not appear", Span::new(0, 0));
            }

            fn should_run_on_file(&self, _source_text: &str, _file_path: &Path) -> bool {
                false
            }
        }

        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(SkipOnceRule)];
        let diags = run_traversal("const x = 1;", &rules);
        assert!(
            diags.is_empty(),
            "run_once rule with should_run_on_file=false should not fire"
        );
    }
}
