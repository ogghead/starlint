//! Example starlint WASM plugin for JSX linting.
//!
//! Demonstrates the JSX AST node support by implementing:
//! - `jsx-example/img-alt-text`: Require `alt` attribute on `<img>` elements.
//! - `jsx-example/no-target-blank`: Require `rel="noreferrer"` when using `target="_blank"`.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    AstNode, Category, LintDiagnostic, NodeBatch, NodeInterest, PluginConfig, RuleMeta, Severity,
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

    fn get_node_interests() -> NodeInterest {
        NodeInterest::JSX_OPENING_ELEMENT
    }

    fn get_file_patterns() -> Vec<String> {
        // Only lint JSX/TSX files.
        vec!["*.jsx".into(), "*.tsx".into()]
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(batch: NodeBatch) -> Vec<LintDiagnostic> {
        let mut diagnostics = Vec::new();

        for node in &batch.nodes {
            if let AstNode::JsxElement(el) = node {
                // Rule: img-alt-text
                if el.name == "img" {
                    let has_alt = el
                        .attributes
                        .iter()
                        .any(|a| !a.is_spread && a.name == "alt");
                    let has_spread = el.attributes.iter().any(|a| a.is_spread);

                    // Only flag if no `alt` and no spread (spread might include alt)
                    if !has_alt && !has_spread {
                        diagnostics.push(LintDiagnostic {
                            rule_name: "jsx-example/img-alt-text".into(),
                            message: "`<img>` elements must have an `alt` attribute".into(),
                            span: el.span,
                            severity: Severity::Error,
                            help: Some(
                                "Add an `alt` attribute describing the image, or `alt=\"\"` for decorative images".into(),
                            ),
                        });
                    }
                }

                // Rule: no-target-blank
                if el.name == "a" {
                    let has_target_blank = el.attributes.iter().any(|a| {
                        !a.is_spread
                            && a.name == "target"
                            && a.value.as_deref() == Some("_blank")
                    });

                    if has_target_blank {
                        let has_rel_noreferrer = el.attributes.iter().any(|a| {
                            !a.is_spread
                                && a.name == "rel"
                                && a.value
                                    .as_deref()
                                    .is_some_and(|v| v.contains("noreferrer"))
                        });

                        if !has_rel_noreferrer {
                            diagnostics.push(LintDiagnostic {
                                rule_name: "jsx-example/no-target-blank".into(),
                                message:
                                    "Using `target=\"_blank\"` without `rel=\"noreferrer\"` is a security risk"
                                        .into(),
                                span: el.span,
                                severity: Severity::Warning,
                                help: Some(
                                    "Add `rel=\"noreferrer\"` to prevent reverse tabnabbing"
                                        .into(),
                                ),
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

export!(JsxExamplePlugin);
