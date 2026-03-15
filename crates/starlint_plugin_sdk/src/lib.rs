//! Shared types for starlint plugins.
//!
//! This crate defines the types shared between the linter core and plugins.
//! It intentionally has no dependency on oxc to keep the plugin API stable.

pub mod diagnostic;
pub mod rule;

pub use diagnostic::{Diagnostic, Edit, Fix, Label, Severity, Span, parse_severity};
pub use rule::{Category, FixKind, RuleMeta};
