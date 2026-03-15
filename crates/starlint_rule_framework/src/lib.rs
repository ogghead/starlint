//! Rule authoring framework for starlint.
//!
//! Provides the [`LintRule`] trait, [`LintContext`], [`Plugin`] trait,
//! [`LintRulePlugin`] adapter, AST traversal dispatch, and fix utilities.
//!
//! This crate is the rule-authoring layer above [`starlint_plugin_sdk`]:
//! plugin crates depend on this for rule infrastructure, while the SDK
//! provides the wire types (diagnostics, spans, rule metadata).

pub mod diagnostic_builder;
pub mod fix;
pub mod fix_builder;
pub mod fix_utils;
pub mod lint_rule;
pub mod lint_rule_plugin;
pub mod macros;
pub mod plugin;
pub mod traversal;

pub use diagnostic_builder::DiagnosticBuilder;
pub use fix::apply_fixes;
pub use fix_builder::FixBuilder;
pub use lint_rule::{LintContext, LintRule};
pub use lint_rule_plugin::LintRulePlugin;
pub use plugin::{FileContext, Plugin};
pub use traversal::{LintDispatchTable, traverse_ast_tree};

// Re-export lint_source when test-utils is enabled.
#[cfg(any(feature = "test-utils", test))]
pub use lint_rule::lint_source;
