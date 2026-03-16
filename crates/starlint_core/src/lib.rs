//! Core linting engine for starlint.
//!
//! Provides the linting engine, file discovery, diagnostic formatting,
//! and overrides. Rule authoring infrastructure is in [`starlint_rule_framework`].

pub mod cache;
pub mod diagnostic;
pub mod directives;
pub mod engine;
#[allow(unused_assignments)] // False positive from thiserror 2.x macro-generated Display impls
pub mod error;
pub mod file_discovery;
pub mod overrides;

pub use engine::{FileDiagnostics, LintSession};
