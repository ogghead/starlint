//! WASM runtime using wasmtime.
//!
//! Manages the wasmtime engine, store, and plugin instances.

/// Resource limits for WASM plugins.
pub struct ResourceLimits {
    /// Maximum fuel (instruction count) per file per plugin.
    pub fuel_per_file: u64,
    /// Maximum memory in bytes.
    pub max_memory_bytes: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            fuel_per_file: 10_000_000,
            max_memory_bytes: 16 * 1024 * 1024, // 16 MB
        }
    }
}

#[cfg(feature = "wasm")]
/// WASM plugin host powered by wasmtime.
pub struct WasmPluginHost {
    /// Resource limits for plugins.
    _limits: ResourceLimits,
}

#[cfg(feature = "wasm")]
impl WasmPluginHost {
    /// Create a new WASM plugin host.
    #[must_use]
    pub const fn new(limits: ResourceLimits) -> Self {
        Self { _limits: limits }
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(
            limits.fuel_per_file, 10_000_000,
            "default fuel should be 10M"
        );
        assert_eq!(
            limits.max_memory_bytes,
            16 * 1024 * 1024,
            "default memory should be 16MB"
        );
    }
}
