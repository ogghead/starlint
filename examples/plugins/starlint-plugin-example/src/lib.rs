//! Example starlint WASM plugin.
//!
//! Implements two rules:
//! - `example/no-debugger`: Flags `debugger` statements and offers a fix to remove them.
//! - `example/no-import-star`: Flags wildcard imports (`import * as X from 'Y'`).

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    Category, Edit, FileContext, Fix, FixKind, Label, LintDiagnostic, PluginConfig, RuleMeta,
    Severity, Span,
};

struct ExamplePlugin;

impl Guest for ExamplePlugin {
    fn get_rules() -> Vec<RuleMeta> {
        vec![
            RuleMeta {
                name: "example/no-debugger".into(),
                description: "Disallow `debugger` statements".into(),
                category: Category::Correctness,
                default_severity: Severity::Error,
            },
            RuleMeta {
                name: "example/no-import-star".into(),
                description: "Disallow wildcard imports".into(),
                category: Category::Style,
                default_severity: Severity::Warning,
            },
        ]
    }

    fn get_file_patterns() -> Vec<String> {
        // Empty = match all files (this example plugin has no file scope restriction).
        Vec::new()
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(file: FileContext, tree: Vec<u8>) -> Vec<LintDiagnostic> {
        // Deserialize the AST tree from JSON bytes.
        let tree: serde_json::Value = match serde_json::from_slice(&tree) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        let mut diagnostics = Vec::new();

        // Walk the nodes array looking for relevant node types.
        if let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) {
            for node in nodes {
                // Check for DebuggerStatement nodes.
                if let Some(debugger) = node.get("DebuggerStatement") {
                    if let Some(span) = extract_span(debugger) {
                        diagnostics.push(LintDiagnostic {
                            rule_name: "example/no-debugger".into(),
                            message: "Unexpected `debugger` statement".into(),
                            span,
                            severity: Severity::Error,
                            help: Some(
                                "Remove the `debugger` statement before committing".into(),
                            ),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Remove `debugger` statement".into(),
                                edits: vec![Edit {
                                    span,
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![Label {
                                span,
                                message: "debugger statement here".into(),
                            }],
                        });
                    }
                }

                // Check for ImportDeclaration with wildcard specifiers.
                if let Some(import) = node.get("ImportDeclaration") {
                    check_import_star(import, &file, &mut diagnostics);
                }
            }
        }

        diagnostics
    }
}

/// Check if an import declaration has wildcard imports.
///
/// Uses a source-text heuristic: looks for `* as` within the import span.
/// This demonstrates that plugins have access to source text via `FileContext`.
fn check_import_star(
    import: &serde_json::Value,
    file: &FileContext,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    let source_module = import
        .get("source")
        .and_then(|s| s.as_str())
        .unwrap_or("<unknown>");
    let span = extract_span(import).unwrap_or(Span { start: 0, end: 0 });

    // Extract the import statement text and look for "* as" pattern.
    let start = span.start as usize;
    let end = span.end as usize;
    let import_text = file
        .source_text
        .get(start..end.min(file.source_text.len()))
        .unwrap_or("");

    if import_text.contains("* as") {
        diagnostics.push(LintDiagnostic {
            rule_name: "example/no-import-star".into(),
            message: format!("Unexpected wildcard import from '{source_module}'"),
            span,
            severity: Severity::Warning,
            help: Some("Import specific exports instead of using `* as`".into()),
            fix: None,
            labels: vec![],
        });
    }
}

/// Extract a WIT Span from a JSON node's "span" field.
fn extract_span(node: &serde_json::Value) -> Option<Span> {
    let span = node.get("span")?;
    let start = span.get("start")?.as_u64()?;
    let end = span.get("end")?.as_u64()?;
    Some(Span {
        start: start as u32,
        end: end as u32,
    })
}

export!(ExamplePlugin);
