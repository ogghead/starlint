//! Example starlint WASM plugin.
//!
//! Implements two rules:
//! - `example/no-debugger`: Flags `debugger` statements.
//! - `example/no-import-star`: Flags wildcard imports (`import * as X from 'Y'`).

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    AstNode, Category, LintDiagnostic, NodeBatch, NodeInterest, PluginConfig, RuleMeta, Severity,
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

    fn get_node_interests() -> NodeInterest {
        NodeInterest::DEBUGGER_STATEMENT | NodeInterest::IMPORT_DECLARATION
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(batch: NodeBatch) -> Vec<LintDiagnostic> {
        let mut diagnostics = Vec::new();

        for node in &batch.nodes {
            match node {
                AstNode::DebuggerStmt(stmt) => {
                    diagnostics.push(LintDiagnostic {
                        rule_name: "example/no-debugger".into(),
                        message: "Unexpected `debugger` statement".into(),
                        span: stmt.span,
                        severity: Severity::Error,
                        help: Some("Remove the `debugger` statement before committing".into()),
                    });
                }
                AstNode::ImportDecl(import) => {
                    // Check for namespace imports: specifier with imported == Some("*")
                    for spec in &import.specifiers {
                        if spec.imported.as_deref() == Some("*") {
                            diagnostics.push(LintDiagnostic {
                                rule_name: "example/no-import-star".into(),
                                message: format!(
                                    "Unexpected wildcard import from '{}'",
                                    import.source
                                ),
                                span: import.span,
                                severity: Severity::Warning,
                                help: Some(
                                    "Import only the specific members you need".into(),
                                ),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        diagnostics
    }
}

export!(ExamplePlugin);
