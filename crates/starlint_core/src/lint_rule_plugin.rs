//! Adapter that wraps existing [`LintRule`] implementations as a [`Plugin`].
//!
//! Preserves single-pass traversal with interest-based dispatch,
//! `run_once` support, `should_run_on_file` filtering, and scope data
//! threading. All ~700 existing rules work through the unified [`Plugin`]
//! interface with zero behavioral change.

use starlint_plugin_sdk::diagnostic::Diagnostic;
use starlint_plugin_sdk::rule::RuleMeta;

use crate::lint_rule::LintRule;
use crate::plugin::{FileContext, Plugin};
use crate::traversal::{LintDispatchTable, traverse_ast_tree};

/// A [`Plugin`] backed by a collection of [`LintRule`] implementations.
///
/// Internally uses the same per-node dispatch table and interest-based
/// filtering as the original engine. This is the backward-compatibility
/// bridge: native rules authored against [`LintRule`] are wrapped here
/// without any code changes.
pub struct LintRulePlugin {
    /// The wrapped lint rules.
    rules: Vec<Box<dyn LintRule>>,
    /// Pre-built dispatch table mapping `AstNodeType` to rule indices.
    dispatch_table: LintDispatchTable,
    /// Indices of rules that only run via `run_once` (no traversal).
    run_once_indices: Vec<usize>,
    /// Whether any wrapped rule needs scope analysis.
    needs_semantic: bool,
}

impl LintRulePlugin {
    /// Create a new adapter wrapping the given lint rules.
    ///
    /// Pre-computes the dispatch table and traversal/run-once partitions.
    #[must_use]
    pub fn new(rules: Vec<Box<dyn LintRule>>) -> Self {
        let needs_semantic = rules.iter().any(|r| r.needs_semantic());

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
        let dispatch_table = LintDispatchTable::build_from_indices(&rules, &traversal_indices);

        Self {
            rules,
            dispatch_table,
            run_once_indices,
            needs_semantic,
        }
    }
}

impl Plugin for LintRulePlugin {
    fn rules(&self) -> Vec<RuleMeta> {
        self.rules.iter().map(|r| r.meta()).collect()
    }

    fn lint_file(&self, ctx: &FileContext<'_>) -> Vec<Diagnostic> {
        traverse_ast_tree(
            ctx.tree,
            &self.rules,
            &self.dispatch_table,
            &self.run_once_indices,
            ctx.source_text,
            ctx.file_path,
            ctx.scope_data,
        )
    }

    fn needs_scope_analysis(&self) -> bool {
        self.needs_semantic
    }

    fn configure(&mut self, config: &str) -> Vec<String> {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(config) else {
            return vec![format!("invalid JSON: {config}")];
        };
        let mut errors = Vec::new();
        for rule in &mut self.rules {
            if let Err(err) = rule.configure(&json) {
                errors.push(err);
            }
        }
        errors
    }
}
