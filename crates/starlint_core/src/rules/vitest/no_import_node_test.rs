//! Rule: `vitest/no-import-node-test`
//!
//! Error when `node:test` is imported. In a Vitest project, importing from
//! the built-in Node.js test runner is almost certainly a mistake. Tests
//! should use Vitest's test runner instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/no-import-node-test";

/// Error when `node:test` is imported.
#[derive(Debug)]
pub struct NoImportNodeTest;

impl NativeRule for NoImportNodeTest {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow importing from `node:test` in Vitest projects".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source_value = import.source.value.as_str();

        if source_value == "node:test" {
            // Replace the import source string literal (including quotes)
            let source_span = Span::new(import.source.span.start, import.source.span.end);
            // Determine the quote character used in the source
            let src = ctx.source_text();
            let quote = src
                .as_bytes()
                .get(usize::try_from(import.source.span.start).unwrap_or(0))
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoImportNodeTest)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

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
