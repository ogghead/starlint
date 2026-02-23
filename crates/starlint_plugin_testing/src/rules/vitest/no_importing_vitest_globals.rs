//! Rule: `vitest/no-importing-vitest-globals`
//!
//! Warn when Vitest globals (`describe`, `it`, `test`, `expect`, `vi`,
//! `beforeEach`, `afterEach`, `beforeAll`, `afterAll`) are explicitly imported
//! from the `vitest` package. When `globals: true` is set in Vitest config,
//! these are available without import.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/no-importing-vitest-globals";

/// Global names that Vitest provides when `globals: true`.
const VITEST_GLOBALS: &[&str] = &[
    "describe",
    "it",
    "test",
    "expect",
    "vi",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
    "suite",
    "bench",
];

/// Warn when Vitest globals are explicitly imported from `vitest`.
#[derive(Debug)]
pub struct NoImportingVitestGlobals;

impl LintRule for NoImportingVitestGlobals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Disallow explicit imports of Vitest globals when `globals: true` is configured"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("from \"vitest\"") || source_text.contains("from 'vitest'"))
            && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        let source_value = import.source.as_str();

        if source_value != "vitest" {
            return;
        }

        // Check if any imported specifier is a known Vitest global.
        // specifiers is Box<[NodeId]> — resolve each through ctx.node()
        if import.specifiers.is_empty() {
            return;
        }

        // Collect diagnostics data first (to avoid borrow conflicts)
        let diag_data: Vec<(String, starlint_ast::types::Span)> = import
            .specifiers
            .iter()
            .filter_map(|&spec_id| {
                let AstNode::ImportSpecifier(named) = ctx.node(spec_id)? else {
                    return None;
                };
                let imported_name = named.imported.as_str();
                VITEST_GLOBALS
                    .contains(&imported_name)
                    .then(|| (imported_name.to_owned(), named.span))
            })
            .collect();

        for (imported_name, span) in diag_data {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Do not import `{imported_name}` from `vitest` — it is available as a global when `globals: true` is configured"
                ),
                span: Span::new(span.start, span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoImportingVitestGlobals)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_describe_import() {
        let source = r#"import { describe, it, expect } from "vitest";"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 3, "all three Vitest globals should be flagged");
    }

    #[test]
    fn test_allows_non_global_imports() {
        let source = r#"import { expectTypeOf } from "vitest";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-global Vitest imports should not be flagged"
        );
    }

    #[test]
    fn test_allows_imports_from_other_packages() {
        let source = r#"import { describe } from "mocha";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "imports from other packages should not be flagged"
        );
    }
}
