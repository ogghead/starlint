//! Rule: `no-use-before-define` (eslint)
//!
//! Disallow the use of variables before they are defined. This helps
//! avoid confusion and ensures code reads top-to-bottom.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags references to variables used before their declaration.
#[derive(Debug)]
pub struct NoUseBeforeDefine;

impl LintRule for NoUseBeforeDefine {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-use-before-define".to_owned(),
            description: "Disallow use of variables before they are defined".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_scope_analysis(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Check let/const declarations — they have a temporal dead zone
        let AstNode::VariableDeclaration(decl) = node else {
            return;
        };

        // Only let/const have TDZ issues
        if decl.kind == VariableDeclarationKind::Var {
            return;
        }

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

                // Check if any reference to this symbol comes before the declaration
                for reference in scope_data.get_resolved_references(symbol_id) {
                    if reference.span.start < span.start {
                        ctx.report(Diagnostic {
                            rule_name: "no-use-before-define".to_owned(),
                            message: format!("'{name}' is used before it is defined"),
                            span: Span::new(reference.span.start, reference.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
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

    starlint_rule_framework::lint_rule_test!(NoUseBeforeDefine);

    #[test]
    fn test_allows_use_after_define() {
        let diags = lint("const x = 1; foo(x);");
        assert!(diags.is_empty(), "use after define should not be flagged");
    }

    #[test]
    fn test_allows_var_hoisting() {
        let diags = lint("foo(x); var x = 1;");
        assert!(diags.is_empty(), "var hoisting should not be flagged");
    }
}
