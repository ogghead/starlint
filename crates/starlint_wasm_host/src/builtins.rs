//! Embedded WASM bytes for builtin plugins.
//!
//! Each builtin plugin is compiled to a `.wasm` component and embedded in the
//! binary via `include_bytes!`. This avoids the need for users to distribute
//! `.wasm` files alongside the starlint binary.

/// Embedded WASM bytes for each builtin plugin, keyed by WASM plugin name.
const BUILTIN_PLUGINS: &[(&str, &[u8])] = &[
    (
        "storybook",
        include_bytes!("../../../tests/fixtures/plugins/storybook-plugin.wasm"),
    ),
    (
        "testing",
        include_bytes!("../../../tests/fixtures/plugins/testing-plugin.wasm"),
    ),
    (
        "react",
        include_bytes!("../../../tests/fixtures/plugins/react-plugin.wasm"),
    ),
    (
        "modules",
        include_bytes!("../../../tests/fixtures/plugins/modules-plugin.wasm"),
    ),
    (
        "nextjs",
        include_bytes!("../../../tests/fixtures/plugins/nextjs-plugin.wasm"),
    ),
    (
        "vue",
        include_bytes!("../../../tests/fixtures/plugins/vue-plugin.wasm"),
    ),
    (
        "jsdoc",
        include_bytes!("../../../tests/fixtures/plugins/jsdoc-plugin.wasm"),
    ),
    (
        "typescript",
        include_bytes!("../../../tests/fixtures/plugins/typescript-plugin.wasm"),
    ),
];

/// Look up embedded WASM bytes for a builtin plugin by WASM name.
///
/// Returns `None` if the name is not a known builtin.
pub fn get_builtin_bytes(wasm_name: &str) -> Option<&'static [u8]> {
    BUILTIN_PLUGINS
        .iter()
        .find(|(name, _)| *name == wasm_name)
        .map(|(_, bytes)| *bytes)
}

/// Map a config-level builtin name to its WASM plugin name.
///
/// The config uses fine-grained category names (`import`, `node`, `promise`)
/// but these three share a single `modules` plugin. This function handles
/// that mapping and returns `None` for unknown names.
pub fn config_to_wasm_name(config_name: &str) -> Option<&'static str> {
    match config_name {
        "storybook" => Some("storybook"),
        "testing" => Some("testing"),
        "react" => Some("react"),
        "nextjs" => Some("nextjs"),
        "vue" => Some("vue"),
        "jsdoc" => Some("jsdoc"),
        "typescript" => Some("typescript"),
        "import" | "node" | "promise" | "modules" => Some("modules"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_bytes_known() {
        assert!(
            get_builtin_bytes("storybook").is_some(),
            "storybook should have embedded bytes"
        );
        assert!(
            get_builtin_bytes("modules").is_some(),
            "modules should have embedded bytes"
        );
    }

    #[test]
    fn test_get_builtin_bytes_unknown() {
        assert!(
            get_builtin_bytes("nonexistent").is_none(),
            "unknown plugin should return None"
        );
    }

    #[test]
    fn test_config_to_wasm_name_direct() {
        assert_eq!(config_to_wasm_name("react"), Some("react"));
        assert_eq!(config_to_wasm_name("typescript"), Some("typescript"));
    }

    #[test]
    fn test_config_to_wasm_name_modules_mapping() {
        assert_eq!(config_to_wasm_name("import"), Some("modules"));
        assert_eq!(config_to_wasm_name("node"), Some("modules"));
        assert_eq!(config_to_wasm_name("promise"), Some("modules"));
        assert_eq!(config_to_wasm_name("modules"), Some("modules"));
    }

    #[test]
    fn test_config_to_wasm_name_unknown() {
        assert_eq!(config_to_wasm_name("nonexistent"), None);
    }
}
