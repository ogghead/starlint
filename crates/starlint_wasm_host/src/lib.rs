//! WASM plugin host for starlint.
//!
//! Loads and runs WASM plugins using the wasmtime runtime.
//! Plugins communicate via the WIT-defined interface in `wit/plugin.wit`.

pub mod builtins;
#[allow(unused_assignments)] // False positive from thiserror 2.x macro-generated Display impls
pub mod error;
pub mod loader;
pub mod runtime;
