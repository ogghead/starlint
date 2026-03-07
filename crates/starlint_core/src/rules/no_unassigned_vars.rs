//! Rule: `no-unassigned-vars`
//!
//! Flag `let` declarations without initializers. A `let x;` declaration
//! leaves the variable as `undefined` and is often a sign of incomplete
//! code. Prefer `let x = <value>;` or use `const` when possible.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;

/// Flags `let` declarators that have no initializer.
#[derive(Debug)]
pub struct NoUnassignedVars;

impl LintRule for NoUnassignedVars {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unassigned-vars".to_owned(),
            description: "Disallow `let` declarations without an initializer".to_owned(),
            category: Category::Suggestion,
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

        // Only flag `let` declarations -- `var` is legacy, `const` requires an init.
        if decl.kind != VariableDeclarationKind::Let {
            return;
        }

        for declarator_id in &*decl.declarations {
            let Some(AstNode::VariableDeclarator(declarator)) = ctx.node(*declarator_id) else {
                continue;
            };

            // Skip if there is an initializer
            if declarator.init.is_some() {
                continue;
            }

            // Only flag simple binding identifiers (not destructured patterns)
            let Some(AstNode::BindingIdentifier(ident)) = ctx.node(declarator.id) else {
                continue;
            };

            let name = ident.name.as_str();

            ctx.report(Diagnostic {
                rule_name: "no-unassigned-vars".to_owned(),
                message: format!("Variable `{name}` is declared with `let` but has no initializer"),
                span: Span::new(declarator.span.start, declarator.span.end),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnassignedVars)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_let_without_init() {
        let diags = lint("let x;");
        assert_eq!(diags.len(), 1, "let without initializer should be flagged");
    }

    #[test]
    fn test_allows_let_with_init() {
        let diags = lint("let x = 1;");
        assert!(
            diags.is_empty(),
            "let with initializer should not be flagged"
        );
    }

    #[test]
    fn test_allows_var_without_init() {
        let diags = lint("var x;");
        assert!(
            diags.is_empty(),
            "var without initializer should not be flagged (only checks let)"
        );
    }

    #[test]
    fn test_allows_const_with_init() {
        let diags = lint("const x = 1;");
        assert!(
            diags.is_empty(),
            "const with initializer should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_declarators() {
        let diags = lint("let a, b;");
        assert_eq!(
            diags.len(),
            2,
            "both declarators without initializers should be flagged"
        );
    }

    #[test]
    fn test_flags_only_uninitialised() {
        let diags = lint("let a = 1, b;");
        assert_eq!(
            diags.len(),
            1,
            "only the declarator without init should be flagged"
        );
    }

    #[test]
    fn test_allows_destructured_without_init() {
        // Destructured patterns without init are a syntax error in practice,
        // but the rule only flags simple identifiers -- skip destructured.
        let diags = lint("let [a, b] = [1, 2];");
        assert!(
            diags.is_empty(),
            "destructured with init should not be flagged"
        );
    }
}
