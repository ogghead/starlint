//! WASM host error types.

use miette::Diagnostic;
use thiserror::Error;

/// Errors from WASM plugin operations.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum WasmError {
    /// Plugin file could not be loaded.
    #[error("failed to load plugin {path}: {reason}")]
    #[diagnostic(
        code(starlint::wasm::load),
        help("Check the plugin path in starlint.toml")
    )]
    LoadFailed {
        /// Plugin file path.
        path: String,
        /// Error details from the OS.
        reason: String,
    },

    /// WASM engine or component compilation error.
    #[error("WASM compilation failed for {path}: {reason}")]
    #[diagnostic(
        code(starlint::wasm::compile),
        help("Ensure the plugin is a valid WASM component")
    )]
    CompileFailed {
        /// Plugin file path.
        path: String,
        /// Error details.
        reason: String,
    },

    /// Plugin instantiation or call failed.
    #[error("WASM runtime error in plugin '{plugin_name}': {reason}")]
    #[diagnostic(code(starlint::wasm::runtime))]
    RuntimeError {
        /// Plugin name.
        plugin_name: String,
        /// Error details.
        reason: String,
    },

    /// Plugin configuration error.
    #[error("plugin '{plugin_name}' rejected configuration: {errors}")]
    #[diagnostic(code(starlint::wasm::config))]
    ConfigRejected {
        /// Plugin name.
        plugin_name: String,
        /// Validation errors from the plugin.
        errors: String,
    },
}
