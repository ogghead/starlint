//! Proof-of-concept lint rules using the unified [`LintRule`] trait.
//!
//! These rules use [`AstTree`] instead of oxc's `AstKind` and serve as
//! validation that the new trait + dispatch system produces identical
//! diagnostics to the legacy `NativeRule` implementations.

pub mod eqeqeq;
pub mod no_debugger;
pub mod no_empty;
pub mod no_var;
