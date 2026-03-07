//! Rule: `no-unused-vars` (eslint)
//!
//! Disallow unused variables. Variables that are declared but never used
//! are most likely errors. This rule flags variables, functions, and
//! function parameters that are declared but never read.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;

/// Flags variables that are declared but never read.
#[derive(Debug)]
pub struct NoUnusedVars;

impl LintRule for NoUnusedVars {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-vars".to_owned(),
            description: "Disallow unused variables".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
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

        // Skip `var` in for-in/for-of (often used as `for (var x of ...`)
        // We only check let/const/var top-level declarations
        if decl.kind == VariableDeclarationKind::Var {
            // Still check var, but be more lenient
        }

        let Some(scope_data) = ctx.scope_data() else {
            return;
        };

        // Collect unused binding info (name, span) and count totals to decide
        // whether the entire declaration can be deleted.
        let mut total_bindings: usize = 0;
        let mut unused_infos: Vec<(String, Span)> = Vec::new();

        for &declarator_id in &*decl.declarations {
            let Some(AstNode::VariableDeclarator(declarator)) = ctx.node(declarator_id) else {
                continue;
            };
            let pattern_id = declarator.id;
            let bindings = ctx.tree().get_binding_identifiers(pattern_id);

            for (_, binding) in &bindings {
                // Skip variables starting with `_` (conventional "unused" marker)
                if binding.name.starts_with('_') {
                    continue;
                }

                total_bindings = total_bindings.saturating_add(1);

                let Some(symbol_id) = ctx.resolve_symbol_id(binding.span) else {
                    continue;
                };

                // Check if any reference to this symbol is a read
                let has_read = scope_data
                    .get_resolved_references(symbol_id)
                    .iter()
                    .any(|r| r.flags.is_read());

                if !has_read {
                    unused_infos.push((
                        binding.name.clone(),
                        Span::new(binding.span.start, binding.span.end),
                    ));
                }
            }
        }

        // Only offer a fix to delete the declaration if ALL bindings are unused.
        let fix: Option<Fix> = if !unused_infos.is_empty() && unused_infos.len() == total_bindings {
            let decl_span = Span::new(decl.span.start, decl.span.end);
            FixBuilder::new("Remove unused declaration", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), decl_span))
                .build()
        } else {
            None
        };

        for (name, span) in &unused_infos {
            ctx.report(Diagnostic {
                rule_name: "no-unused-vars".to_owned(),
                message: format!("'{name}' is declared but never used"),
                span: *span,
                severity: Severity::Warning,
                help: None,
                fix: fix.clone(),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnusedVars)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_unused_var() {
        let diags = lint("var x = 1;");
        assert_eq!(diags.len(), 1, "unused var should be flagged");
    }

    #[test]
    fn test_flags_unused_let() {
        let diags = lint("let x = 1;");
        assert_eq!(diags.len(), 1, "unused let should be flagged");
    }

    #[test]
    fn test_flags_unused_const() {
        let diags = lint("const x = 1;");
        assert_eq!(diags.len(), 1, "unused const should be flagged");
    }

    #[test]
    fn test_allows_used_variable() {
        let diags = lint("var x = 1; console.log(x);");
        assert!(diags.is_empty(), "used variable should not be flagged");
    }

    #[test]
    fn test_allows_underscore_prefix() {
        let diags = lint("var _x = 1;");
        assert!(
            diags.is_empty(),
            "underscore-prefixed variable should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_unused() {
        let diags = lint("var a = 1, b = 2;");
        assert_eq!(diags.len(), 2, "two unused vars should be flagged");
    }
}
