//! Rule: `sort-imports`
//!
//! Require import declarations to be sorted alphabetically by their source
//! module specifier. Only checks the order of import declarations, not the
//! members within each import.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::{AstNode, ImportDeclarationNode};
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags import declarations that are not sorted by source specifier.
#[derive(Debug)]
pub struct SortImports;

impl LintRule for SortImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "sort-imports".to_owned(),
            description: "Require import declarations to be sorted".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Program])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Program(program) = node else {
            return;
        };

        // Collect all import declarations in order
        let imports: Vec<&ImportDeclarationNode> = program
            .body
            .iter()
            .filter_map(|stmt_id| {
                if let Some(AstNode::ImportDeclaration(import)) = ctx.node(*stmt_id) {
                    Some(import)
                } else {
                    None
                }
            })
            .collect();

        if imports.len() < 2 {
            return;
        }

        // Check pairwise ordering by source specifier (case-insensitive)
        for pair in imports.windows(2) {
            let Some(prev) = pair.first().copied() else {
                continue;
            };
            let Some(curr) = pair.get(1).copied() else {
                continue;
            };

            let prev_source = prev.source.as_str();
            let curr_source = curr.source.as_str();

            if prev_source.to_lowercase() > curr_source.to_lowercase() {
                ctx.report(Diagnostic {
                    rule_name: "sort-imports".to_owned(),
                    message: format!(
                        "Import from '{curr_source}' should come before import from '{prev_source}'"
                    ),
                    span: Span::new(curr.span.start, curr.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                // Report only the first violation
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(SortImports);

    #[test]
    fn test_allows_sorted_imports() {
        let diags = lint("import a from 'a';\nimport b from 'b';\nimport c from 'c';");
        assert!(diags.is_empty(), "sorted imports should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_imports() {
        let diags = lint("import b from 'b';\nimport a from 'a';");
        assert_eq!(diags.len(), 1, "unsorted imports should be flagged");
    }

    #[test]
    fn test_allows_single_import() {
        let diags = lint("import a from 'a';");
        assert!(diags.is_empty(), "single import should not be flagged");
    }

    #[test]
    fn test_case_insensitive_sort() {
        let diags = lint("import a from 'Alpha';\nimport b from 'beta';");
        assert!(diags.is_empty(), "case-insensitive sort should pass");
    }

    #[test]
    fn test_flags_case_insensitive_unsorted() {
        let diags = lint("import b from 'beta';\nimport a from 'Alpha';");
        assert_eq!(
            diags.len(),
            1,
            "case-insensitive unsorted should be flagged"
        );
    }

    #[test]
    fn test_allows_no_imports() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "no imports should not be flagged");
    }
}
