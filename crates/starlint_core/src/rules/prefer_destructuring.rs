//! Rule: `prefer-destructuring`
//!
//! Require destructuring from arrays and objects when accessing a specific
//! element or property directly. For example, prefer `const { x } = obj`
//! over `const x = obj.x`.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;

/// Flags variable declarations that could use destructuring.
#[derive(Debug)]
pub struct PreferDestructuring;

impl LintRule for PreferDestructuring {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-destructuring".to_owned(),
            description: "Prefer destructuring from arrays and objects".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclaration])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclaration(decl) = node else {
            return;
        };

        // Only check const/let declarations (not var — legacy code patterns)
        if decl.kind == VariableDeclarationKind::Var {
            return;
        }

        for &declarator_id in &*decl.declarations {
            let Some(AstNode::VariableDeclarator(declarator)) = ctx.node(declarator_id) else {
                continue;
            };

            // Must be a simple identifier binding (not already destructured)
            let binding_name = ctx.node(declarator.id).and_then(|n| {
                if let AstNode::BindingIdentifier(ident) = n {
                    Some(ident.name.clone())
                } else {
                    None
                }
            });

            let Some(binding_name) = binding_name else {
                continue;
            };

            let Some(init_id) = declarator.init else {
                continue;
            };

            let declarator_span = declarator.span;

            let Some(init_node) = ctx.node(init_id) else {
                continue;
            };

            // Check if init is a member expression like `obj.prop` or `arr[0]`
            match init_node {
                AstNode::StaticMemberExpression(member) => {
                    let prop_name = &member.property;

                    // Only suggest if the variable name matches the property name
                    if binding_name == *prop_name {
                        ctx.report(Diagnostic {
                            rule_name: "prefer-destructuring".to_owned(),
                            message: format!("Use object destructuring: `{{ {prop_name} }} = ...`"),
                            span: Span::new(declarator_span.start, declarator_span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
                AstNode::ComputedMemberExpression(member) => {
                    // arr[0] — suggest destructuring for numeric indices
                    if ctx
                        .node(member.expression)
                        .is_some_and(|n| matches!(n, AstNode::NumericLiteral(_)))
                    {
                        ctx.report(Diagnostic {
                            rule_name: "prefer-destructuring".to_owned(),
                            message: "Use array destructuring instead of indexed access".to_owned(),
                            span: Span::new(declarator_span.start, declarator_span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferDestructuring)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_object_property_access() {
        let diags = lint("const x = obj.x;");
        assert_eq!(
            diags.len(),
            1,
            "same-name property access should be flagged"
        );
    }

    #[test]
    fn test_allows_different_name() {
        // const y = obj.x — names differ, not a simple destructuring target
        let diags = lint("const y = obj.x;");
        assert!(diags.is_empty(), "different name should not be flagged");
    }

    #[test]
    fn test_allows_already_destructured() {
        let diags = lint("const { x } = obj;");
        assert!(
            diags.is_empty(),
            "already destructured should not be flagged"
        );
    }

    #[test]
    fn test_allows_var() {
        let diags = lint("var x = obj.x;");
        assert!(diags.is_empty(), "var should not be checked");
    }

    #[test]
    fn test_flags_array_index() {
        let diags = lint("const x = arr[0];");
        assert_eq!(diags.len(), 1, "indexed access should be flagged");
    }

    #[test]
    fn test_allows_computed_non_numeric() {
        let diags = lint("const x = obj[key];");
        assert!(
            diags.is_empty(),
            "computed non-numeric should not be flagged"
        );
    }
}
