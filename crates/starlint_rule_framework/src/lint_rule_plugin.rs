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

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use std::path::Path;

    use starlint_plugin_sdk::diagnostic::Severity;
    use starlint_plugin_sdk::rule::{Category, RuleMeta};

    use starlint_ast::node_type::AstNodeType;

    /// Minimal mock lint rule for testing.
    #[derive(Debug)]
    struct MockRule {
        /// Name for identification.
        name: &'static str,
        /// Whether this rule needs traversal.
        traversal: bool,
        /// Whether this rule needs scope analysis.
        semantic: bool,
    }

    impl MockRule {
        /// Create a traversal-based rule.
        const fn traversal(name: &'static str) -> Self {
            Self {
                name,
                traversal: true,
                semantic: false,
            }
        }

        /// Create a run-once rule.
        const fn once_only(name: &'static str) -> Self {
            Self {
                name,
                traversal: false,
                semantic: false,
            }
        }

        /// Create a rule needing scope analysis.
        const fn with_semantic(name: &'static str) -> Self {
            Self {
                name,
                traversal: true,
                semantic: true,
            }
        }
    }

    impl LintRule for MockRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: self.name.to_owned(),
                description: String::new(),
                category: Category::Correctness,
                default_severity: Severity::Error,
            }
        }

        fn needs_traversal(&self) -> bool {
            self.traversal
        }

        fn needs_semantic(&self) -> bool {
            self.semantic
        }

        fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
            None
        }
    }

    #[test]
    fn test_new_empty_rules() {
        let plugin = LintRulePlugin::new(vec![]);
        assert!(
            plugin.rules().is_empty(),
            "empty plugin should have no rules"
        );
        assert!(
            !plugin.needs_scope_analysis(),
            "empty plugin should not need scope analysis"
        );
    }

    #[test]
    fn test_rules_returns_metadata() {
        let rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(MockRule::traversal("test-rule-a")),
            Box::new(MockRule::traversal("test-rule-b")),
        ];
        let plugin = LintRulePlugin::new(rules);
        let metas = plugin.rules();
        assert_eq!(metas.len(), 2, "should return metadata for all rules");
        assert_eq!(
            metas.first().map(|m| m.name.as_str()),
            Some("test-rule-a"),
            "first rule name"
        );
        assert_eq!(
            metas.get(1).map(|m| m.name.as_str()),
            Some("test-rule-b"),
            "second rule name"
        );
    }

    #[test]
    fn test_needs_scope_analysis_false() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MockRule::traversal("no-semantic"))];
        let plugin = LintRulePlugin::new(rules);
        assert!(
            !plugin.needs_scope_analysis(),
            "should not need scope analysis when no rule requires it"
        );
    }

    #[test]
    fn test_needs_scope_analysis_true() {
        let rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(MockRule::traversal("basic")),
            Box::new(MockRule::with_semantic("semantic")),
        ];
        let plugin = LintRulePlugin::new(rules);
        assert!(
            plugin.needs_scope_analysis(),
            "should need scope analysis when any rule requires it"
        );
    }

    #[test]
    fn test_run_once_partitioning() {
        let rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(MockRule::traversal("traversal-rule")),
            Box::new(MockRule::once_only("once-rule")),
            Box::new(MockRule::traversal("another-traversal")),
        ];
        let plugin = LintRulePlugin::new(rules);
        assert_eq!(
            plugin.run_once_indices.len(),
            1,
            "should have one run-once rule"
        );
        assert_eq!(
            plugin.run_once_indices.first().copied(),
            Some(1),
            "run-once index should be 1"
        );
    }

    #[test]
    fn test_configure_invalid_json() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MockRule::traversal("test"))];
        let mut plugin = LintRulePlugin::new(rules);
        let errors = plugin.configure("not valid json {{{");
        assert_eq!(errors.len(), 1, "should return one error for invalid JSON");
        assert!(
            errors
                .first()
                .is_some_and(|e| e.starts_with("invalid JSON:")),
            "error should indicate invalid JSON"
        );
    }

    #[test]
    fn test_configure_valid_json() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MockRule::traversal("test"))];
        let mut plugin = LintRulePlugin::new(rules);
        let errors = plugin.configure("{}");
        assert!(
            errors.is_empty(),
            "valid JSON should produce no errors with default configure"
        );
    }

    #[test]
    fn test_lint_file_empty_tree() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MockRule::traversal("test"))];
        let plugin = LintRulePlugin::new(rules);
        let tree = starlint_ast::tree::AstTree::new();
        let ctx = FileContext {
            tree: &tree,
            source_text: "",
            file_path: Path::new("test.js"),
            extension: "js",
            scope_data: None,
        };
        let diagnostics = plugin.lint_file(&ctx);
        assert!(
            diagnostics.is_empty(),
            "empty tree should produce no diagnostics"
        );
    }
}
