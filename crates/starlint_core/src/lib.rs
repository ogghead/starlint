//! Core linting engine for starlint.
//!
//! Provides parsing, AST traversal, rule dispatch, file discovery,
//! and diagnostic formatting.

pub mod diagnostic;
pub mod engine;
#[allow(unused_assignments)] // False positive from thiserror 2.x macro-generated Display impls
pub mod error;
pub mod file_discovery;
pub mod fix;
pub mod fix_builder;
pub mod fix_utils;
pub mod lint_rule;
pub mod lint_rule_plugin;
pub mod lint_rules;
pub mod overrides;
pub mod plugin;
pub mod rules;
pub mod traversal;

pub use engine::{FileDiagnostics, LintSession};
pub use plugin::{FileContext, Plugin};
pub use starlint_plugin_sdk;
