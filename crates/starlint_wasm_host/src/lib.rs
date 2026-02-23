//! WASM plugin host for starlint.
//!
//! Loads and runs WASM plugins using the wasmtime runtime.
//! Plugins communicate via the WIT-defined interface in `wit/plugin.wit`.

pub mod bridge;
pub mod error;
pub mod loader;
pub mod runtime;
