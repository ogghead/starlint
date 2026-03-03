//! Integration tests for the WASM plugin host.
//!
//! These tests load a real WASM component (the example plugin) and verify
//! the full pipeline: load → collect nodes → call plugin → get diagnostics.

#![cfg(feature = "wasm")]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::missing_docs_in_private_items,
    clippy::missing_assert_message
)]

use std::path::Path;

use oxc_allocator::Allocator;

use starlint_core::parser::parse_file;
use starlint_core::plugin::PluginHost;
use starlint_wasm_host::runtime::{ResourceLimits, WasmPluginHost};

/// Path to the pre-built example plugin component.
const EXAMPLE_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/example-plugin.wasm"
);

/// Path to the pre-built JSX example plugin component.
const JSX_EXAMPLE_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/jsx-example-plugin.wasm"
);

/// Helper to create a host with the example plugin loaded.
fn host_with_example_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(EXAMPLE_PLUGIN), "")
        .expect("should load example plugin");
    host
}

/// Helper to create a host with the JSX example plugin loaded.
fn host_with_jsx_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(JSX_EXAMPLE_PLUGIN), "")
        .expect("should load JSX example plugin");
    host
}

#[test]
fn test_load_example_plugin() {
    let host = host_with_example_plugin();
    // If we get here without panic, the plugin loaded successfully.
    drop(host);
}

#[test]
fn test_debugger_statement_detected() {
    let host = host_with_example_plugin();
    let allocator = Allocator::default();
    let source = "debugger;";
    let path = Path::new("test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert_eq!(diags.len(), 1, "should detect one debugger statement");

    let first = diags.first().expect("should have a diagnostic");
    assert_eq!(
        first.rule_name, "example/no-debugger",
        "rule name should be example/no-debugger"
    );
    assert!(
        first.message.contains("debugger"),
        "message should mention debugger"
    );
}

#[test]
fn test_import_star_detected() {
    let host = host_with_example_plugin();
    let allocator = Allocator::default();
    let source = "import * as utils from 'utils';";
    let path = Path::new("test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert_eq!(diags.len(), 1, "should detect one wildcard import");

    let first = diags.first().expect("should have a diagnostic");
    assert_eq!(
        first.rule_name, "example/no-import-star",
        "rule name should be example/no-import-star"
    );
    assert!(
        first.message.contains("wildcard"),
        "message should mention wildcard"
    );
}

#[test]
fn test_named_import_not_flagged() {
    let host = host_with_example_plugin();
    let allocator = Allocator::default();
    let source = "import { foo, bar } from 'utils';";
    let path = Path::new("test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "named imports should not be flagged, got: {diags:?}"
    );
}

#[test]
fn test_default_import_not_flagged() {
    let host = host_with_example_plugin();
    let allocator = Allocator::default();
    let source = "import utils from 'utils';";
    let path = Path::new("test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "default imports should not be flagged, got: {diags:?}"
    );
}

#[test]
fn test_multiple_issues_detected() {
    let host = host_with_example_plugin();
    let allocator = Allocator::default();
    let source = "import * as all from 'mod';\ndebugger;\nimport { ok } from 'other';";
    let path = Path::new("test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert_eq!(
        diags.len(),
        2,
        "should detect wildcard import and debugger, got: {diags:?}"
    );

    let rule_names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        rule_names.contains(&"example/no-debugger"),
        "should contain no-debugger diagnostic"
    );
    assert!(
        rule_names.contains(&"example/no-import-star"),
        "should contain no-import-star diagnostic"
    );
}

#[test]
fn test_clean_file_no_diagnostics() {
    let host = host_with_example_plugin();
    let allocator = Allocator::default();
    let source = "const x = 1;\nconst y = x + 2;\nconsole.log(y);";
    let path = Path::new("test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "clean file should have no diagnostics, got: {diags:?}"
    );
}

#[test]
fn test_invalid_plugin_path_fails() {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    let result = host.load_plugin(Path::new("/nonexistent/plugin.wasm"), "");
    assert!(result.is_err(), "loading nonexistent plugin should fail");
}

#[test]
fn test_fuel_exhaustion() {
    let limits = ResourceLimits {
        fuel_per_file: 1, // Extremely low fuel — plugin should run out
        max_memory_bytes: 16 * 1024 * 1024,
    };
    let mut host = WasmPluginHost::new(limits).expect("should create WASM host");

    // Loading should work (uses its own store with default-ish fuel for metadata queries).
    // Actually, load_plugin creates a store with the same limits, so with fuel=1 it may fail
    // during metadata query. Let's just verify the behavior is graceful.
    let result = host.load_plugin(Path::new(EXAMPLE_PLUGIN), "");
    // With only 1 fuel, the metadata query will likely fail.
    assert!(
        result.is_err(),
        "loading plugin with 1 fuel should fail during metadata query"
    );
}

// ---- JSX plugin integration tests ----

#[test]
fn test_load_jsx_plugin() {
    let host = host_with_jsx_plugin();
    drop(host);
}

#[test]
fn test_jsx_img_missing_alt() {
    let host = host_with_jsx_plugin();
    let allocator = Allocator::default();
    let source = r#"const el = <img src="photo.jpg" />;"#;
    let path = Path::new("test.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert_eq!(diags.len(), 1, "should detect missing alt on img");

    let first = diags.first().expect("should have a diagnostic");
    assert_eq!(first.rule_name, "jsx-example/img-alt-text");
    assert!(first.message.contains("alt"), "message should mention alt");
}

#[test]
fn test_jsx_img_with_alt_ok() {
    let host = host_with_jsx_plugin();
    let allocator = Allocator::default();
    let source = r#"const el = <img src="photo.jpg" alt="A photo" />;"#;
    let path = Path::new("test.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "img with alt should not be flagged, got: {diags:?}"
    );
}

#[test]
fn test_jsx_img_with_spread_ok() {
    let host = host_with_jsx_plugin();
    let allocator = Allocator::default();
    let source = r#"const el = <img src="photo.jpg" {...props} />;"#;
    let path = Path::new("test.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "img with spread should not be flagged (spread may include alt), got: {diags:?}"
    );
}

#[test]
fn test_jsx_target_blank_without_noreferrer() {
    let host = host_with_jsx_plugin();
    let allocator = Allocator::default();
    let source = r#"const el = <a href="https://example.com" target="_blank">Link</a>;"#;
    let path = Path::new("test.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert_eq!(
        diags.len(),
        1,
        "should detect target=_blank without noreferrer"
    );

    let first = diags.first().expect("should have a diagnostic");
    assert_eq!(first.rule_name, "jsx-example/no-target-blank");
}

#[test]
fn test_jsx_target_blank_with_noreferrer_ok() {
    let host = host_with_jsx_plugin();
    let allocator = Allocator::default();
    let source =
        r#"const el = <a href="https://example.com" target="_blank" rel="noreferrer">Link</a>;"#;
    let path = Path::new("test.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "target=_blank with noreferrer should not be flagged, got: {diags:?}"
    );
}

#[test]
fn test_jsx_non_img_not_flagged() {
    let host = host_with_jsx_plugin();
    let allocator = Allocator::default();
    let source = r#"const el = <div className="container"><span>Hello</span></div>;"#;
    let path = Path::new("test.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse should succeed");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "non-img elements should not be flagged, got: {diags:?}"
    );
}
