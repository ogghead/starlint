//! Rule: `sort-vars`
//!
//! Require variables within the same declaration to be sorted alphabetically.
//! For example, `var a, b, c;` is valid, but `var b, a, c;` is not.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags variable declarations where declarators are not alphabetically sorted.
#[derive(Debug)]
pub struct SortVars;

impl LintRule for SortVars {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "sort-vars".to_owned(),
            description: "Require variables within the same declaration to be sorted".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclaration(decl) = node else {
            return;
        };

        // Only check multi-declarator statements: `var a, b, c;`
        if decl.declarations.len() < 2 {
            return;
        }

        // Only check var/let/const with simple identifier bindings
        // Skip destructuring patterns — ordering doesn't apply there
        let names: Vec<(String, starlint_ast::types::Span)> = decl
            .declarations
            .iter()
            .filter_map(|&d_id| {
                let AstNode::VariableDeclarator(d) = ctx.node(d_id)? else {
                    return None;
                };
                let AstNode::BindingIdentifier(ident) = ctx.node(d.id)? else {
                    return None;
                };
                Some((ident.name.clone(), ident.span))
            })
            .collect();

        if names.len() < 2 {
            return;
        }

        // Check pairwise ordering (case-insensitive by default, matching ESLint)
        for pair in names.windows(2) {
            let Some((prev_name, _)) = pair.first() else {
                continue;
            };
            let Some((curr_name, curr_span)) = pair.get(1) else {
                continue;
            };

            if prev_name.to_lowercase() > curr_name.to_lowercase() {
                ctx.report(Diagnostic {
                    rule_name: "sort-vars".to_owned(),
                    message: format!(
                        "Variables within the same declaration should be sorted alphabetically. \
                         Expected '{curr_name}' to come before '{prev_name}'"
                    ),
                    span: Span::new(curr_span.start, curr_span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                // Report only the first violation per declaration
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(SortVars);

    #[test]
    fn test_allows_sorted_vars() {
        let diags = lint("var a, b, c;");
        assert!(diags.is_empty(), "sorted vars should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_vars() {
        let diags = lint("var b, a;");
        assert_eq!(diags.len(), 1, "unsorted vars should be flagged");
    }

    #[test]
    fn test_allows_single_var() {
        let diags = lint("var a;");
        assert!(diags.is_empty(), "single var should not be flagged");
    }

    #[test]
    fn test_allows_sorted_let() {
        let diags = lint("let alpha, beta, gamma;");
        assert!(diags.is_empty(), "sorted let should not be flagged");
    }

    #[test]
    fn test_flags_unsorted_const() {
        let diags = lint("const z = 1, a = 2;");
        assert_eq!(diags.len(), 1, "unsorted const should be flagged");
    }

    #[test]
    fn test_case_insensitive() {
        let diags = lint("var a, B, c;");
        assert!(diags.is_empty(), "case-insensitive sorting should pass");
    }

    #[test]
    fn test_flags_case_insensitive_unsorted() {
        let diags = lint("var B, a;");
        assert_eq!(
            diags.len(),
            1,
            "case-insensitive unsorted should be flagged"
        );
    }

    #[test]
    fn test_separate_declarations_independent() {
        let diags = lint("var b; var a;");
        assert!(
            diags.is_empty(),
            "separate declarations should not affect each other"
        );
    }
}
