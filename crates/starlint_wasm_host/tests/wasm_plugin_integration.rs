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

/// Path to the pre-built storybook plugin component.
const STORYBOOK_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/storybook-plugin.wasm"
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

// ---- Storybook plugin integration tests ----

fn host_with_storybook_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(STORYBOOK_PLUGIN), "")
        .expect("should load storybook plugin");
    host
}

#[test]
fn test_load_storybook_plugin() {
    let host = host_with_storybook_plugin();
    drop(host);
}

#[test]
fn test_storybook_default_exports_missing() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    let source = "export const Primary = {};";
    let path = Path::new("Button.stories.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"storybook/default-exports"),
        "should flag missing default export, got: {names:?}"
    );
}

#[test]
fn test_storybook_default_exports_present() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    let source = "export default { component: Button };\nexport const Primary = {};";
    let path = Path::new("Button.stories.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        !names.contains(&"storybook/default-exports"),
        "should NOT flag when default export present, got: {names:?}"
    );
}

#[test]
fn test_storybook_story_exports_missing() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    let source = "export default { component: Button };";
    let path = Path::new("Button.stories.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"storybook/story-exports"),
        "should flag missing named exports, got: {names:?}"
    );
}

#[test]
fn test_storybook_no_stories_of() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    let source = "export default {};\nexport const Primary = {};\nstoriesOf('Button', module);";
    let path = Path::new("Button.stories.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"storybook/no-stories-of"),
        "should flag storiesOf usage, got: {names:?}"
    );
}

#[test]
fn test_storybook_use_storybook_testing_library() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    let source = "import { render } from '@testing-library/react';\nexport default { component: Button };\nexport const Primary = {};";
    let path = Path::new("Button.stories.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"storybook/use-storybook-testing-library"),
        "should flag @testing-library import, got: {names:?}"
    );
}

#[test]
fn test_storybook_hierarchy_separator() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    let source = "export default { component: Button, title: 'Components|Button' };\nexport const Primary = {};";
    let path = Path::new("Button.stories.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"storybook/hierarchy-separator"),
        "should flag | separator in title, got: {names:?}"
    );
}

#[test]
fn test_storybook_file_pattern_skip() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    // Same source but NOT a stories file — should be skipped.
    let source = "storiesOf('Button', module);";
    let path = Path::new("Button.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "non-story files should be skipped entirely, got: {diags:?}"
    );
}

#[test]
fn test_storybook_clean_story() {
    let host = host_with_storybook_plugin();
    let allocator = Allocator::default();
    let source = r#"
import { Button } from './Button';
export default { component: Button };
export const Primary = {};
export const Secondary = { args: { variant: 'secondary' } };
"#;
    let path = Path::new("Button.stories.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "clean story should have no diagnostics, got: {diags:?}"
    );
}

// ---- Testing plugin integration tests ----

/// Path to the pre-built testing plugin component.
const TESTING_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/testing-plugin.wasm"
);

fn host_with_testing_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(TESTING_PLUGIN), "")
        .expect("should load testing plugin");
    host
}

#[test]
fn test_load_testing_plugin() {
    let host = host_with_testing_plugin();
    drop(host);
}

#[test]
fn test_testing_no_disabled_tests() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "describe('suite', () => { xit('should work', () => {}); });";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jest/no-disabled-tests"),
        "should flag disabled test (xit), got: {names:?}"
    );
}

#[test]
fn test_testing_no_focused_tests() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "describe.only('suite', () => { it('focused', () => { expect(1).toBe(1); }); });";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jest/no-focused-tests"),
        "should flag focused test (describe.only), got: {names:?}"
    );
}

#[test]
fn test_testing_no_mocks_import() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "import foo from './__mocks__/bar';\ndescribe('test', () => { it('works', () => { expect(foo).toBe(1); }); });";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jest/no-mocks-import"),
        "should flag __mocks__ import, got: {names:?}"
    );
}

#[test]
fn test_testing_vitest_no_import_node_test() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "import { test } from 'node:test';\ndescribe('suite', () => { it('works', () => { expect(1).toBe(1); }); });";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"vitest/no-import-node-test"),
        "should flag node:test import, got: {names:?}"
    );
}

#[test]
fn test_testing_no_commented_out_tests() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "// it('should work', () => {});\ndescribe('suite', () => { it('works', () => { expect(1).toBe(1); }); });";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jest/no-commented-out-tests"),
        "should flag commented out test, got: {names:?}"
    );
}

#[test]
fn test_testing_no_export() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "const helper = () => {};\nexport { helper };\ndescribe('suite', () => { it('works', () => { expect(helper()).toBe(1); }); });";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jest/no-export"),
        "should flag exports from test file, got: {names:?}"
    );
}

#[test]
fn test_testing_file_pattern_skip() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "const x = 1;";
    let path = Path::new("src/utils.js"); // Not a test file
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "non-test file should have no diagnostics, got: {diags:?}"
    );
}

#[test]
fn test_testing_consistent_test_it() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "describe('suite', () => {\n  it('one', () => { expect(1).toBe(1); });\n  test('two', () => { expect(2).toBe(2); });\n});";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jest/consistent-test-it"),
        "should flag inconsistent test/it usage, got: {names:?}"
    );
}

#[test]
fn test_testing_vitest_prefer_to_be_truthy() {
    let host = host_with_testing_plugin();
    let allocator = Allocator::default();
    let source = "describe('suite', () => {\n  it('checks true', () => { expect(foo).toBe(true); });\n});";
    let path = Path::new("foo.test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"vitest/prefer-to-be-truthy"),
        "should flag toBe(true), got: {names:?}"
    );
}

// ---- React plugin integration tests ----

/// Path to the pre-built react plugin component.
const REACT_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/react-plugin.wasm"
);

fn host_with_react_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(REACT_PLUGIN), "")
        .expect("should load react plugin");
    host
}

#[test]
fn test_load_react_plugin() {
    let host = host_with_react_plugin();
    drop(host);
}

#[test]
fn test_react_jsx_no_target_blank() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "export default function App() { return <a href=\"https://example.com\" target=\"_blank\">Link</a>; }";
    let path = Path::new("App.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"react/jsx-no-target-blank"),
        "should flag target=_blank without rel, got: {names:?}"
    );
}

#[test]
fn test_react_a11y_alt_text() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "export default function App() { return <img src=\"photo.jpg\" />; }";
    let path = Path::new("App.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsx-a11y/alt-text"),
        "should flag img without alt, got: {names:?}"
    );
}

#[test]
fn test_react_a11y_html_has_lang() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "export default function Page() { return <html><body>Hello</body></html>; }";
    let path = Path::new("Page.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsx-a11y/html-has-lang"),
        "should flag html without lang, got: {names:?}"
    );
}

#[test]
fn test_react_button_has_type() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "export default function App() { return <button onClick={handleClick}>Click</button>; }";
    let path = Path::new("App.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"react/button-has-type"),
        "should flag button without type, got: {names:?}"
    );
}

#[test]
fn test_react_no_danger() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "export default function App() { return <div dangerouslySetInnerHTML={{__html: '<b>hi</b>'}} />; }";
    let path = Path::new("App.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"react/no-danger"),
        "should flag dangerouslySetInnerHTML, got: {names:?}"
    );
}

#[test]
fn test_react_file_pattern_skip() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "const x = 1;";
    let path = Path::new("utils.js"); // Not a .jsx/.tsx file
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        diags.is_empty(),
        "non-JSX file should have no diagnostics, got: {diags:?}"
    );
}

#[test]
fn test_react_iframe_missing_sandbox() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "export default function App() { return <iframe src=\"https://example.com\" />; }";
    let path = Path::new("App.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"react/iframe-missing-sandbox"),
        "should flag iframe without sandbox, got: {names:?}"
    );
}

#[test]
fn test_react_a11y_click_events_have_key_events() {
    let host = host_with_react_plugin();
    let allocator = Allocator::default();
    let source = "export default function App() { return <div onClick={handleClick}>Click me</div>; }";
    let path = Path::new("App.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsx-a11y/click-events-have-key-events"),
        "should flag onClick without keyboard handler, got: {names:?}"
    );
}

// ---- Modules plugin integration tests ----

/// Path to the pre-built modules plugin component.
const MODULES_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/modules-plugin.wasm"
);

fn host_with_modules_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(MODULES_PLUGIN), "")
        .expect("should load modules plugin");
    host
}

#[test]
fn test_load_modules_plugin() {
    let host = host_with_modules_plugin();
    drop(host);
}

#[test]
fn test_import_no_duplicates() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "import { foo } from 'lodash';\nimport { bar } from 'lodash';\n";
    let path = Path::new("index.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"import/no-duplicates"),
        "should flag duplicate imports from 'lodash', got: {names:?}"
    );
}

#[test]
fn test_import_no_default_export() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "export default function main() {}";
    let path = Path::new("utils.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"import/no-default-export"),
        "should flag default export, got: {names:?}"
    );
}

#[test]
fn test_import_no_namespace() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "import * as utils from './utils';";
    let path = Path::new("app.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"import/no-namespace"),
        "should flag namespace import, got: {names:?}"
    );
}

#[test]
fn test_node_no_process_exit() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "process.exit(1);";
    let path = Path::new("server.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"node/no-process-exit"),
        "should flag process.exit(), got: {names:?}"
    );
}

#[test]
fn test_promise_prefer_await_to_then() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "fetch('/api').then(data => data.json());";
    let path = Path::new("api.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"promise/prefer-await-to-then"),
        "should flag .then() usage, got: {names:?}"
    );
}

#[test]
fn test_promise_no_nesting() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "fetch('/a').then(() => { return fetch('/b').then(() => {}); });";
    let path = Path::new("nested.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"promise/no-nesting"),
        "should flag nested promise, got: {names:?}"
    );
}

#[test]
fn test_import_no_nodejs_modules() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "import punycode from 'punycode';";
    let path = Path::new("legacy.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"import/no-nodejs-modules"),
        "should flag bare Node.js module import, got: {names:?}"
    );
}

#[test]
fn test_modules_all_files_no_pattern_skip() {
    let host = host_with_modules_plugin();
    let allocator = Allocator::default();
    let source = "export default function main() {}";
    let path = Path::new("deeply/nested/component.ts");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    assert!(
        !diags.is_empty(),
        "modules plugin should lint ALL files (no file pattern filter)"
    );
}

// ---- Next.js plugin integration tests ----

/// Path to the pre-built nextjs plugin component.
const NEXTJS_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/nextjs-plugin.wasm"
);

fn host_with_nextjs_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(NEXTJS_PLUGIN), "")
        .expect("should load nextjs plugin");
    host
}

#[test]
fn test_load_nextjs_plugin() {
    let host = host_with_nextjs_plugin();
    drop(host);
}

#[test]
fn test_nextjs_no_img_element() {
    let host = host_with_nextjs_plugin();
    let allocator = Allocator::default();
    let source = "export default function Page() { return <img src=\"/photo.jpg\" />; }";
    let path = Path::new("page.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"nextjs/no-img-element"),
        "should flag <img> element, got: {names:?}"
    );
}

#[test]
fn test_nextjs_no_head_element() {
    let host = host_with_nextjs_plugin();
    let allocator = Allocator::default();
    let source = "export default function Page() { return <head><title>Hi</title></head>; }";
    let path = Path::new("page.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"nextjs/no-head-element"),
        "should flag <head> element, got: {names:?}"
    );
}

#[test]
fn test_nextjs_no_html_link_for_pages() {
    let host = host_with_nextjs_plugin();
    let allocator = Allocator::default();
    let source = "export default function Nav() { return <a href=\"/about\">About</a>; }";
    let path = Path::new("nav.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"nextjs/no-html-link-for-pages"),
        "should flag <a href='/about'>, got: {names:?}"
    );
}

#[test]
fn test_nextjs_no_sync_scripts() {
    let host = host_with_nextjs_plugin();
    let allocator = Allocator::default();
    let source = "export default function Page() { return <script src=\"/analytics.js\"></script>; }";
    let path = Path::new("page.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"nextjs/no-sync-scripts"),
        "should flag sync script, got: {names:?}"
    );
}

#[test]
fn test_nextjs_no_document_import_in_page() {
    let host = host_with_nextjs_plugin();
    let allocator = Allocator::default();
    let source = "import Document from 'next/document';\nexport default function Page() { return <div />; }";
    let path = Path::new("page.jsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"nextjs/no-document-import-in-page"),
        "should flag next/document import outside _document, got: {names:?}"
    );
}

#[test]
fn test_nextjs_no_async_client_component() {
    let host = host_with_nextjs_plugin();
    let allocator = Allocator::default();
    let source = "\"use client\";\nexport default async function Page() { return <div />; }";
    let path = Path::new("page.tsx");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"nextjs/no-async-client-component"),
        "should flag async client component, got: {names:?}"
    );
}

// ---- Vue plugin integration tests ----

/// Path to the pre-built vue plugin component.
const VUE_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/vue-plugin.wasm"
);

fn host_with_vue_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(VUE_PLUGIN), "")
        .expect("should load vue plugin");
    host
}

#[test]
fn test_load_vue_plugin() {
    let host = host_with_vue_plugin();
    drop(host);
}

#[test]
fn test_vue_no_arrow_functions_in_watch() {
    let host = host_with_vue_plugin();
    let allocator = Allocator::default();
    let source = "export default { watch: { count: (val) => console.log(val) } }";
    let path = Path::new("Counter.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"vue/no-arrow-functions-in-watch"),
        "should flag arrow function in watch, got: {names:?}"
    );
}

#[test]
fn test_vue_no_async_in_computed() {
    let host = host_with_vue_plugin();
    let allocator = Allocator::default();
    let source = "export default { computed: { async fetchedData() { return await fetch('/api'); } } }";
    let path = Path::new("Data.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"vue/no-async-in-computed-properties"),
        "should flag async in computed, got: {names:?}"
    );
}

#[test]
fn test_vue_no_child_content_with_v_html() {
    let host = host_with_vue_plugin();
    let allocator = Allocator::default();
    let source = "const template = `<div v-html=\"rawHtml\">Some text</div>`;";
    let path = Path::new("Unsafe.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"vue/no-child-content"),
        "should flag v-html with child content, got: {names:?}"
    );
}

#[test]
fn test_vue_no_ref_reactivity_loss() {
    let host = host_with_vue_plugin();
    let allocator = Allocator::default();
    let source = "import { ref } from 'vue';\nconst { value } = ref(42);";
    let path = Path::new("setup.ts");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"vue/no-ref-object-reactivity-loss"),
        "should flag ref destructuring, got: {names:?}"
    );
}

#[test]
fn test_vue_no_component_options_typo() {
    let host = host_with_vue_plugin();
    let allocator = Allocator::default();
    let source = "export default { methdos: { fetchData() {} } }";
    let path = Path::new("Component.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"vue/no-component-options-typo"),
        "should flag 'methdos' typo, got: {names:?}"
    );
}

// ---- JSDoc plugin integration tests ----

/// Path to the pre-built jsdoc plugin component.
const JSDOC_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/jsdoc-plugin.wasm"
);

fn host_with_jsdoc_plugin() -> WasmPluginHost {
    let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("should create WASM host");
    host.load_plugin(Path::new(JSDOC_PLUGIN), "")
        .expect("should load jsdoc plugin");
    host
}

#[test]
fn test_load_jsdoc_plugin() {
    let host = host_with_jsdoc_plugin();
    drop(host);
}

#[test]
fn test_jsdoc_check_tag_names() {
    let host = host_with_jsdoc_plugin();
    let allocator = Allocator::default();
    let source = "/** @foobar This is a test */\nfunction test() {}";
    let path = Path::new("utils.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsdoc/check-tag-names"),
        "should flag unknown @foobar tag, got: {names:?}"
    );
}

#[test]
fn test_jsdoc_check_types() {
    let host = host_with_jsdoc_plugin();
    let allocator = Allocator::default();
    let source = "/** @param {String} name The name */\nfunction greet(name) {}";
    let path = Path::new("greet.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsdoc/check-types"),
        "should flag {{String}} -> use {{string}}, got: {names:?}"
    );
}

#[test]
fn test_jsdoc_require_param_type() {
    let host = host_with_jsdoc_plugin();
    let allocator = Allocator::default();
    let source = "/** @param name The name */\nfunction greet(name) {}";
    let path = Path::new("greet.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsdoc/require-param-type"),
        "should flag missing type in @param, got: {names:?}"
    );
}

#[test]
fn test_jsdoc_no_defaults() {
    let host = host_with_jsdoc_plugin();
    let allocator = Allocator::default();
    let source = "/** @default 42 */\nconst ANSWER = 42;";
    let path = Path::new("constants.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsdoc/no-defaults"),
        "should flag @default tag, got: {names:?}"
    );
}

#[test]
fn test_jsdoc_match_description() {
    let host = host_with_jsdoc_plugin();
    let allocator = Allocator::default();
    let source = "/** lowercase description */\nfunction test() {}";
    let path = Path::new("test.js");
    let parsed = parse_file(&allocator, source, path).expect("parse");

    let diags = host.lint_file(path, source, &parsed.program);
    let names: Vec<&str> = diags.iter().map(|d| d.rule_name.as_str()).collect();
    assert!(
        names.contains(&"jsdoc/match-description"),
        "should flag lowercase description, got: {names:?}"
    );
}


