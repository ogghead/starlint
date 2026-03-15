//! Rule: `init-declarations`
//!
//! Require initialization in variable declarations (default mode: "always").
//! Variables declared without an initializer are a potential source of
//! `undefined`-related bugs. However, for-in and for-of loop variables are
//! assigned implicitly and are exempt.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags variable declarations without initializers.
#[derive(Debug)]
pub struct InitDeclarations;

impl LintRule for InitDeclarations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "init-declarations".to_owned(),
            description: "Require initialization in variable declarations".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclaration])
    }

    fn run(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclaration(decl) = node else {
            return;
        };

        // Check if this declaration is the left-hand side of a for-in/for-of.
        // Walk ancestors to find if parent is a ForInStatement or ForOfStatement.
        if let Some(parent_id) = ctx.parent(node_id) {
            if let Some(AstNode::ForInStatement(_) | AstNode::ForOfStatement(_)) =
                ctx.node(parent_id)
            {
                return;
            }
        }

        // Collect diagnostics from declarators
        let mut diags = Vec::new();
        for &declarator_id in &*decl.declarations {
            if let Some(AstNode::VariableDeclarator(declarator)) = ctx.node(declarator_id) {
                if declarator.init.is_none() {
                    let span = declarator.span;
                    diags.push(Diagnostic {
                        rule_name: "init-declarations".to_owned(),
                        message: "Variable declaration should be initialized".to_owned(),
                        span: Span::new(span.start, span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
        }

        for diag in diags {
            ctx.report(diag);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(InitDeclarations);

    #[test]
    fn test_flags_var_without_init() {
        let diags = lint("var x;");
        assert_eq!(diags.len(), 1, "var without init should be flagged");
    }

    #[test]
    fn test_allows_var_with_init() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "var with init should not be flagged");
    }

    #[test]
    fn test_flags_let_without_init() {
        let diags = lint("let x;");
        assert_eq!(diags.len(), 1, "let without init should be flagged");
    }

    #[test]
    fn test_allows_let_with_init() {
        let diags = lint("let x = 1;");
        assert!(diags.is_empty(), "let with init should not be flagged");
    }

    #[test]
    fn test_allows_const_with_init() {
        let diags = lint("const x = 1;");
        assert!(
            diags.is_empty(),
            "const always has init and should not be flagged"
        );
    }

    #[test]
    fn test_allows_for_in_var() {
        let diags = lint("for (var x in obj) {}");
        assert!(
            diags.is_empty(),
            "for-in variable should not be flagged (implicitly assigned)"
        );
    }

    #[test]
    fn test_allows_for_of_let() {
        let diags = lint("for (let x of arr) {}");
        assert!(
            diags.is_empty(),
            "for-of variable should not be flagged (implicitly assigned)"
        );
    }

    #[test]
    fn test_flags_multiple_uninit_declarators() {
        let diags = lint("var a, b;");
        assert_eq!(
            diags.len(),
            2,
            "two uninitialised declarators should produce two diagnostics"
        );
    }

    #[test]
    fn test_flags_only_uninit_declarator() {
        let diags = lint("var a = 1, b;");
        assert_eq!(
            diags.len(),
            1,
            "only the uninitialized declarator should be flagged"
        );
    }

    #[test]
    fn test_allows_for_of_destructuring() {
        let diags = lint("for (const [a, b] of pairs) {}");
        assert!(
            diags.is_empty(),
            "for-of destructuring should not be flagged"
        );
    }
}
