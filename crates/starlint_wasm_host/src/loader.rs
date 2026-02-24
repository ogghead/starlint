//! Plugin discovery and loading.
//!
//! Loads WASM plugin files from paths specified in config.

use std::path::{Path, PathBuf};

use miette::miette;

/// Validate that a plugin file exists and has a `.wasm` extension.
pub fn validate_plugin_path(path: &Path) -> miette::Result<PathBuf> {
    if !path.exists() {
        return Err(miette!("plugin file not found: {}", path.display()));
    }

    let has_wasm_ext = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext == "wasm");

    if !has_wasm_ext {
        return Err(miette!(
            "plugin file must have .wasm extension, got: {}",
            path.display()
        ));
    }

    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_validate_nonexistent() {
        let result = validate_plugin_path(Path::new("/nonexistent/plugin.wasm"));
        assert!(result.is_err(), "nonexistent file should fail validation");
    }

    #[test]
    fn test_validate_wrong_extension() {
        // Use a file that likely exists but is not a .wasm
        let result = validate_plugin_path(Path::new("/dev/null"));
        assert!(result.is_err(), "non-wasm extension should fail validation");
    }
}
