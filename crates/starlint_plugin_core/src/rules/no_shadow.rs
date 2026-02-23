//! Rule: `no-shadow`
//!
//! Disallow variable declarations from shadowing variables declared in an
//! outer scope. Shadowing can lead to confusion about which variable is
//! being referenced.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags variable declarations that shadow a variable from an outer scope.
#[derive(Debug)]
pub struct NoShadow;

impl LintRule for NoShadow {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-shadow".to_owned(),
            description: "Disallow variable declarations from shadowing variables in outer scope"
                .to_owned(),
            category: Category::Suggestion,
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

        let Some(scope_data) = ctx.scope_data() else {
            return;
        };

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

                // Get the scope of this binding
                let binding_scope = scope_data.symbol_scope_id(symbol_id);

                // Walk up parent scopes looking for a same-named binding
                let mut current_scope = scope_data.scope_parent_id(binding_scope);

                while let Some(scope_id) = current_scope {
                    if scope_data.get_binding(scope_id, name).is_some() {
                        let decl_span = Span::new(span.start, span.end);
                        let new_name = format!("{name}_inner");
                        let fix = {
                            let edits = fix_utils::rename_symbol_edits(
                                scope_data, symbol_id, &new_name, decl_span,
                            );
                            FixBuilder::new(
                                format!("Rename to `{new_name}`"),
                                FixKind::SuggestionFix,
                            )
                            .edits(edits)
                            .build()
                        };
                        ctx.report(Diagnostic {
                            rule_name: "no-shadow".to_owned(),
                            message: format!("'{name}' is already declared in the upper scope"),
                            span: decl_span,
                            severity: Severity::Warning,
                            help: Some(format!("Consider renaming to `{new_name}`")),
                            fix,
                            labels: vec![],
                        });
                        break;
                    }

                    current_scope = scope_data.scope_parent_id(scope_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoShadow)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_shadowed_var() {
        let diags = lint("var x = 1; function foo() { var x = 2; }");
        assert_eq!(diags.len(), 1, "shadowed var should be flagged");
    }

    #[test]
    fn test_flags_shadowed_let() {
        let diags = lint("let x = 1; { let x = 2; }");
        assert_eq!(diags.len(), 1, "shadowed let should be flagged");
    }

    #[test]
    fn test_allows_different_names() {
        let diags = lint("var x = 1; function foo() { var y = 2; }");
        assert!(diags.is_empty(), "different names should not be flagged");
    }

    #[test]
    fn test_allows_same_scope() {
        // Same-scope redeclaration is handled by no-redeclare, not no-shadow
        let diags = lint("var x = 1; var y = 2;");
        assert!(diags.is_empty(), "same scope should not be flagged");
    }

    #[test]
    fn test_nested_shadow() {
        let diags = lint("var x = 1; function foo() { var x = 2; function bar() { var x = 3; } }");
        assert_eq!(diags.len(), 2, "each nested shadow should be flagged");
    }
}
