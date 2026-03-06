//! Stable AST types for starlint.
//!
//! This crate defines a flat, indexed AST representation where nodes reference
//! children by [`NodeId`] index into a shared [`AstTree`] arena. This design
//! sidesteps WIT's inability to express recursive types while providing a
//! zero-copy traversal model for native rules and a serializable format for
//! WASM plugins.
//!
//! **No oxc dependency.** The converter from oxc's arena-allocated AST to
//! [`AstTree`] lives in `starlint_core`.

pub mod node;
pub mod node_type;
pub mod operator;
pub mod traverse;
pub mod tree;
pub mod types;

pub use node::AstNode;
pub use node_type::AstNodeType;
pub use traverse::{TreeCursor, TreeVisitor};
pub use tree::AstTree;
pub use types::{NodeId, Span};
