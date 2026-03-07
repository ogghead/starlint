//! Rule: `no-redeclare`
//!
//! Disallow variable redeclaration within the same scope.
//! Uses semantic analysis to detect when the same name is bound
//! multiple times in a single scope.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags variables that are redeclared in the same scope.
#[derive(Debug)]
pub struct NoRedeclare;

impl LintRule for NoRedeclare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-redeclare".to_owned(),
            description: "Disallow variable redeclaration".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclaration(decl) = node else {
            return;
        };

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        for &declarator_id in &*decl.declarations {
            let pattern_id = match ctx.node(declarator_id) {
                Some(AstNode::VariableDeclarator(d)) => d.id,
                _ => continue,
            };

            // Collect owned binding data to avoid borrow conflicts with ctx.report().
            let binding_data: Vec<(String, starlint_ast::types::Span)> = ctx
                .tree()
                .get_binding_identifiers(pattern_id)
                .iter()
                .map(|(_, b)| (b.name.clone(), b.span))
                .collect();

            for (name, span) in &binding_data {
                let Some(symbol_id) = ctx.resolve_symbol_id(*span) else {
                    continue;
                };

                // Check for redeclarations via the semantic redeclare list
                let redeclarations = scoping.symbol_redeclarations(symbol_id);
                if !redeclarations.is_empty() {
                    // Only report on the redeclaration, not the original
                    // The original declaration's span will differ from the redecl spans
                    for respan in redeclarations {
                        let new_name = format!("{name}_2");
                        let respan_sdk = Span::new(respan.span.start, respan.span.end);
                        let fix = FixBuilder::new(
                            format!("Rename to `{new_name}`"),
                            FixKind::SuggestionFix,
                        )
                        .replace(respan_sdk, &new_name)
                        .build();

                        ctx.report(Diagnostic {
                            rule_name: "no-redeclare".to_owned(),
                            message: format!("'{name}' is already defined"),
                            span: respan_sdk,
                            severity: Severity::Error,
                            help: Some(format!(
                                "Consider renaming to `{new_name}` to avoid redeclaration"
                            )),
                            fix,
                            labels: vec![],
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRedeclare)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_var_redeclaration() {
        let diags = lint("var x = 1; var x = 2;");
        assert!(!diags.is_empty(), "var redeclaration should be flagged");
    }

    #[test]
    fn test_allows_different_names() {
        let diags = lint("var x = 1; var y = 2;");
        assert!(diags.is_empty(), "different names should not be flagged");
    }

    #[test]
    fn test_allows_different_scopes() {
        let diags = lint("var x = 1; function foo() { var x = 2; }");
        assert!(
            diags.is_empty(),
            "different scopes should not be flagged by no-redeclare"
        );
    }

    #[test]
    fn test_allows_let_in_different_blocks() {
        let diags = lint("{ let x = 1; } { let x = 2; }");
        assert!(
            diags.is_empty(),
            "let in different blocks should not be flagged"
        );
    }
}
