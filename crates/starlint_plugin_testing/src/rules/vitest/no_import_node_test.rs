//! Rule: `vitest/no-import-node-test`
//!
//! Error when `node:test` is imported. In a Vitest project, importing from
//! the built-in Node.js test runner is almost certainly a mistake. Tests
//! should use Vitest's test runner instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/no-import-node-test";

/// Error when `node:test` is imported.
#[derive(Debug)]
pub struct NoImportNodeTest;

impl LintRule for NoImportNodeTest {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow importing from `node:test` in Vitest projects".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("node:test") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        let source_value = import.source.as_str();

        if source_value == "node:test" {
            // Replace the import source string literal (including quotes)
            let source_span = Span::new(import.source_span.start, import.source_span.end);
            // Determine the quote character used in the source
            let src = ctx.source_text();
            let quote = src
                .as_bytes()
                .get(usize::try_from(import.source_span.start).unwrap_or(0))
                .copied()
                .unwrap_or(b'"');
            let quote_char = char::from(quote);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not import from `node:test` — use Vitest's test runner instead"
                    .to_owned(),
                span: Span::new(import.span.start, import.span.end),
                severity: Severity::Error,
                help: Some("Replace `node:test` with `vitest`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with `vitest`".to_owned(),
                    edits: vec![Edit {
                        span: source_span,
                        replacement: format!("{quote_char}vitest{quote_char}"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoImportNodeTest);

    #[test]
    fn test_flags_node_test_import() {
        let source = r#"import { test } from "node:test";"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "importing from `node:test` should be flagged"
        );
    }

    #[test]
    fn test_allows_vitest_import() {
        let source = r#"import { test } from "vitest";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "importing from `vitest` should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_node_imports() {
        let source = r#"import fs from "node:fs";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "importing from `node:fs` should not be flagged"
        );
    }
}
