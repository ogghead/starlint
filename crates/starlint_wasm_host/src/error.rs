//! WASM host error types.

use miette::Diagnostic;
use thiserror::Error;

/// Errors from WASM plugin operations.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum WasmError {
    /// Plugin file could not be loaded.
    #[error("failed to load plugin: {path}")]
    #[diagnostic(code(starlint::wasm::load), help("Check the plugin path in starlint.toml"))]
    LoadFailed {
        /// Plugin file path.
        path: String,
    },

    /// Plugin exceeded resource limits.
    #[error("plugin exceeded resource limits: {plugin_name}")]
    #[diagnostic(
        code(starlint::wasm::resource_limit),
        help("The plugin may be in an infinite loop or using too much memory")
    )]
    ResourceLimit {
        /// Plugin name.
        plugin_name: String,
    },

    /// Plugin returned an invalid result.
    #[error("plugin returned invalid result: {plugin_name}")]
    #[diagnostic(code(starlint::wasm::invalid_result))]
    InvalidResult {
        /// Plugin name.
        plugin_name: String,
    },
}
