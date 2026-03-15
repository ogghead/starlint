//! Rule authoring framework for starlint.
//!
//! Provides the [`LintRule`] trait, [`LintContext`], [`Plugin`] trait,
//! [`LintRulePlugin`] adapter, AST traversal dispatch, fix utilities,
//! and shared helpers (AST, JSX, case-conversion, source-text scanning).
//!
//! This crate is the rule-authoring layer above [`starlint_plugin_sdk`]:
//! plugin crates depend on this for rule infrastructure, while the SDK
//! provides the wire types (diagnostics, spans, rule metadata).

pub mod ast_utils;
pub mod case_utils;
pub mod diagnostic_builder;
pub mod fix;
pub mod fix_builder;
pub mod fix_utils;
pub mod jsx_utils;
pub mod lint_rule;
pub mod lint_rule_plugin;
pub mod macros;
pub mod plugin;
pub mod source_utils;
pub mod traversal;

pub use diagnostic_builder::DiagnosticBuilder;

/// Convert an AST [`Span`](starlint_ast::types::Span) to an SDK
/// [`Span`](starlint_plugin_sdk::diagnostic::Span).
///
/// Both span types have identical `start`/`end` fields but live in
/// different crates, so a `From` impl is not possible (orphan rules).
#[must_use]
pub const fn sdk_span(
    ast_span: starlint_ast::types::Span,
) -> starlint_plugin_sdk::diagnostic::Span {
    starlint_plugin_sdk::diagnostic::Span::new(ast_span.start, ast_span.end)
}

pub use fix::apply_fixes;
pub use fix_builder::FixBuilder;
pub use lint_rule::{LintContext, LintRule};
pub use lint_rule_plugin::LintRulePlugin;
pub use plugin::{FileContext, Plugin};
pub use traversal::{LintDispatchTable, traverse_ast_tree};

// Re-export lint_source when test-utils is enabled.
#[cfg(any(feature = "test-utils", test))]
pub use lint_rule::lint_source;
