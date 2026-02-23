//! Rule: `no-const-assign` (eslint)
//!
//! Disallow reassignment of `const` variables. Modifying a constant after
//! declaration causes a runtime error.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};
use starlint_scope::SymbolFlags;

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags reassignment of `const` variables.
#[derive(Debug)]
pub struct NoConstAssign;

impl LintRule for NoConstAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-const-assign".to_owned(),
            description: "Disallow reassignment of const variables".to_owned(),
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

        if decl.kind != VariableDeclarationKind::Const {
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

                let flags = scope_data.symbol_flags(symbol_id);
                if !flags.contains(SymbolFlags::CONST_VARIABLE) {
                    continue;
                }

                // Check if any reference to this symbol is a write
                let has_write = scope_data
                    .get_resolved_references(symbol_id)
                    .iter()
                    .any(|r| r.flags.is_write());

                if has_write {
                    // Suggest changing `const` to `let` so reassignment is valid.
                    let kw_span = Span::new(decl.span.start, decl.span.start.saturating_add(5));
                    let fix = FixBuilder::new("Change `const` to `let`", FixKind::SuggestionFix)
                        .replace(kw_span, "let")
                        .build();

                    ctx.report(Diagnostic {
                        rule_name: "no-const-assign".to_owned(),
                        message: format!("'{name}' is a constant and cannot be reassigned",),
                        span: Span::new(span.start, span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Use `let` instead of `const` if reassignment is intended".to_owned(),
                        ),
                        fix,
                        labels: vec![],
                    });
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConstAssign)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_const_reassignment() {
        let diags = lint("const x = 1; x = 2;");
        assert_eq!(diags.len(), 1, "reassigning const should be flagged");
    }

    #[test]
    fn test_allows_const_read() {
        let diags = lint("const x = 1; console.log(x);");
        assert!(diags.is_empty(), "reading const should not be flagged");
    }

    #[test]
    fn test_allows_let_reassignment() {
        let diags = lint("let x = 1; x = 2;");
        assert!(diags.is_empty(), "reassigning let should not be flagged");
    }

    #[test]
    fn test_flags_const_destructuring_reassignment() {
        let diags = lint("const { a } = obj; a = 2;");
        assert_eq!(
            diags.len(),
            1,
            "reassigning destructured const should be flagged"
        );
    }
}
