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
pub mod overrides;
pub mod parser;
pub mod plugin;
pub mod rule;
pub mod rules;
pub mod traversal;

pub use engine::{FileDiagnostics, LintSession};
pub use starlint_plugin_sdk;
