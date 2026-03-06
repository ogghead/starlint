//! Example starlint WASM plugin for JSX linting (v2 — full AST tree + fix support).
//!
//! Demonstrates the JSX AST node support by implementing:
//! - `jsx-example/img-alt-text`: Require `alt` attribute on `<img>` elements.
//! - `jsx-example/no-target-blank`: Require `rel="noreferrer"` when using `target="_blank"`.

wit_bindgen::generate!({
    world: "linter-plugin-v2",
    path: "wit",
});

use exports::starlint::plugin::plugin_v2::Guest;
use starlint::plugin::types::{
    Category, FileContext, LintDiagnosticV2, PluginConfig, RuleMeta, Severity, Span,
};

struct JsxExamplePlugin;

impl Guest for JsxExamplePlugin {
    fn get_rules() -> Vec<RuleMeta> {
        vec![
            RuleMeta {
                name: "jsx-example/img-alt-text".into(),
                description: "Require `alt` attribute on `<img>` elements".into(),
                category: Category::Correctness,
                default_severity: Severity::Error,
            },
            RuleMeta {
                name: "jsx-example/no-target-blank".into(),
                description: "Require `rel=\"noreferrer\"` when using `target=\"_blank\"`".into(),
                category: Category::Correctness,
                default_severity: Severity::Warning,
            },
        ]
    }

    fn get_file_patterns() -> Vec<String> {
        // Only lint JSX/TSX files.
        vec!["*.jsx".into(), "*.tsx".into()]
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(_file: FileContext, tree: Vec<u8>) -> Vec<LintDiagnosticV2> {
        let tree: serde_json::Value = match serde_json::from_slice(&tree) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        let mut diagnostics = Vec::new();

        if let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) {
            for node in nodes {
                if let Some(jsx) = node.get("JSXOpeningElement") {
                    let name = jsx.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let span = extract_span(jsx).unwrap_or(Span { start: 0, end: 0 });

                    // Rule: img-alt-text
                    if name == "img" {
                        let attrs = jsx.get("attributes").and_then(|a| a.as_array());
                        let mut has_alt = false;
                        let mut has_spread = false;

                        if let Some(attr_ids) = attrs {
                            for attr_id_val in attr_ids {
                                if let Some(attr_id) = attr_id_val.as_u64() {
                                    if let Some((attr_name, _value, is_spread)) =
                                        get_jsx_attr(&tree, attr_id)
                                    {
                                        if is_spread {
                                            has_spread = true;
                                        }
                                        if attr_name == "alt" {
                                            has_alt = true;
                                        }
                                    }
                                }
                            }
                        }

                        // Only flag if no `alt` and no spread (spread might include alt)
                        if !has_alt && !has_spread {
                            diagnostics.push(LintDiagnosticV2 {
                                rule_name: "jsx-example/img-alt-text".into(),
                                message: "`<img>` elements must have an `alt` attribute".into(),
                                span,
                                severity: Severity::Error,
                                help: Some(
                                    "Add an `alt` attribute describing the image, or `alt=\"\"` for decorative images".into(),
                                ),
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }

                    // Rule: no-target-blank
                    if name == "a" {
                        let attrs = jsx.get("attributes").and_then(|a| a.as_array());
                        let mut has_target_blank = false;
                        let mut has_rel_noreferrer = false;

                        if let Some(attr_ids) = attrs {
                            for attr_id_val in attr_ids {
                                if let Some(attr_id) = attr_id_val.as_u64() {
                                    if let Some((attr_name, value, is_spread)) =
                                        get_jsx_attr(&tree, attr_id)
                                    {
                                        if !is_spread
                                            && attr_name == "target"
                                            && value.as_deref() == Some("_blank")
                                        {
                                            has_target_blank = true;
                                        }
                                        if !is_spread
                                            && attr_name == "rel"
                                            && value
                                                .as_deref()
                                                .is_some_and(|v| v.contains("noreferrer"))
                                        {
                                            has_rel_noreferrer = true;
                                        }
                                    }
                                }
                            }
                        }

                        if has_target_blank && !has_rel_noreferrer {
                            diagnostics.push(LintDiagnosticV2 {
                                rule_name: "jsx-example/no-target-blank".into(),
                                message:
                                    "Using `target=\"_blank\"` without `rel=\"noreferrer\"` is a security risk"
                                        .into(),
                                span,
                                severity: Severity::Warning,
                                help: Some(
                                    "Add `rel=\"noreferrer\"` to prevent reverse tabnabbing"
                                        .into(),
                                ),
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                }
            }
        }

        diagnostics
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

fn get_jsx_attr(
    tree: &serde_json::Value,
    attr_id: u64,
) -> Option<(String, Option<String>, bool)> {
    let nodes = tree.get("nodes")?.as_array()?;
    let node = nodes.get(attr_id as usize)?;
    if let Some(attr) = node.get("JSXAttribute") {
        let name = attr.get("name")?.as_str()?.to_string();
        let value = attr.get("value").and_then(|v| {
            if v.is_null() {
                return None;
            }
            let vid = v.as_u64()?;
            let value_node = nodes.get(vid as usize)?;
            if let Some(lit) = value_node.get("StringLiteral") {
                return lit.get("value").and_then(|v| v.as_str()).map(|s| s.to_string());
            }
            None
        });
        return Some((name, value, false));
    }
    if node.get("JSXSpreadAttribute").is_some() {
        return Some(("".to_string(), None, true));
    }
    None
}

export!(JsxExamplePlugin);
