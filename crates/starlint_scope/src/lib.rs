//! Lightweight scope analysis for [`starlint_ast::tree::AstTree`].
//!
//! Provides symbol table, scope tree, reference tracking, and unresolved
//! reference detection — replacing `oxc_semantic` without a second parse.

pub mod builder;
pub mod scope_data;
pub mod types;

pub use builder::build as build_scope_data;
pub use scope_data::ScopeData;
pub use types::{ReferenceFlags, ReferenceInfo, ScopeId, SymbolFlags, SymbolId, UnresolvedRef};
