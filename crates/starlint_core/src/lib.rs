//! Core linting engine for starlint.
//!
//! Provides the linting engine, file discovery, diagnostic formatting,
//! and overrides. Rule authoring infrastructure is in [`starlint_rule_framework`]
//! and re-exported here for backward compatibility.

pub mod diagnostic;
pub mod engine;
#[allow(unused_assignments)] // False positive from thiserror 2.x macro-generated Display impls
pub mod error;
pub mod file_discovery;
pub mod overrides;

// Re-export rule framework modules for backward compatibility.
// Downstream crates (starlint_loader, starlint_wasm_host, starlint_cli)
// can continue importing from starlint_core::{lint_rule, plugin, ...}.
pub use starlint_rule_framework::fix;
pub use starlint_rule_framework::fix_builder;
pub use starlint_rule_framework::fix_utils;
pub use starlint_rule_framework::lint_rule;
pub use starlint_rule_framework::lint_rule_plugin;
pub use starlint_rule_framework::plugin;
pub use starlint_rule_framework::traversal;

pub use engine::{FileDiagnostics, LintSession};
pub use plugin::{FileContext, Plugin};
