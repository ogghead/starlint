//! Integration tests for starlint.

use std::path::Path;

use starlint_core::diagnostic::OutputFormat;
use starlint_core::engine::LintSession;
use starlint_core::file_discovery::discover_files;

#[test]
fn test_lint_session_with_no_rules_produces_no_diagnostics() {
    let session = LintSession::new(vec![], OutputFormat::Pretty);
    let result = session.lint_single_file(Path::new("test.ts"), "const x: number = 1;");
    assert!(
        result.diagnostics.is_empty(),
        "no rules should produce no diagnostics"
    );
}

#[test]
fn test_file_discovery_finds_fixtures() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let files = discover_files(&[fixtures_dir]);
    assert!(
        files.len() >= 2,
        "should find at least 2 fixture files, found: {}",
        files.len()
    );
}

#[test]
fn test_config_deserialize_empty() {
    let config: Result<starlint_config::Config, _> = toml::from_str("");
    assert!(
        config.is_ok(),
        "empty config should deserialize to defaults"
    );
}

#[test]
fn test_config_roundtrip() {
    let toml_str = r#"
[settings]
threads = 4

[rules]
"no-debugger" = "error"
"#;
    let config: Result<starlint_config::Config, _> = toml::from_str(toml_str);
    assert!(config.is_ok(), "config should deserialize");
    if let Ok(cfg) = config {
        assert_eq!(cfg.settings.threads, 4, "threads should be 4");
        assert_eq!(cfg.rules.len(), 1, "should have one rule");
    }
}

#[test]
fn test_parse_valid_typescript() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "export const hello: string = 'world';";
    let result = starlint_core::parser::parse_file(&allocator, source, Path::new("test.ts"));
    assert!(result.is_ok(), "valid TypeScript should parse");
}

#[test]
fn test_parse_valid_jsx() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "const App = () => <div>Hello</div>;";
    let result = starlint_core::parser::parse_file(&allocator, source, Path::new("test.jsx"));
    assert!(result.is_ok(), "valid JSX should parse");
}
