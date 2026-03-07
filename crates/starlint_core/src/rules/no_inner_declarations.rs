//! Rule: `no-inner-declarations`
//!
//! Disallow variable or function declarations in nested blocks.
//! Prior to ES6, function declarations were only allowed in the program body
//! or a function body. While ES6 allows block-level functions in strict mode,
//! `var` declarations in blocks are still hoisted and can be confusing.
//!
//! Uses the parent-chain from `AstTree` to determine whether a declaration is
//! inside a nested block (not directly in a program or function body).

#![allow(clippy::match_same_arms)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;

/// Flags function or `var` declarations inside nested blocks.
#[derive(Debug)]
pub struct NoInnerDeclarations;

/// Check whether the given node is directly inside a valid (non-nested) context
/// by walking the parent chain from the `AstTree`.
///
/// A declaration is valid if its nearest enclosing statement container is:
/// - The program body
/// - A function body (function, arrow, method, etc.)
/// - A static block
///
/// It is invalid (inner) if the nearest container is a block inside an
/// `if`/`for`/`while`/`switch`/etc.
fn is_in_valid_position(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let mut current = node_id;
    while let Some(parent_id) = ctx.parent(current) {
        match ctx.node(parent_id) {
            // Program, function body, or static block -> valid position.
            Some(
                AstNode::Program(_)
                | AstNode::Function(_)
                | AstNode::ArrowFunctionExpression(_)
                | AstNode::StaticBlock(_),
            ) => return true,

            // FunctionBody is just a container, keep going up.
            Some(AstNode::FunctionBody(_)) => {}

            // BlockStatement: check if it's the body of a function or program (keep going)
            // or a nested block under control flow (invalid).
            Some(AstNode::BlockStatement(_)) => {
                // Keep walking - the parent of this block will tell us the context.
            }

            // Control-flow / nesting constructs -> invalid (inner) position.
            Some(
                AstNode::IfStatement(_)
                | AstNode::ForStatement(_)
                | AstNode::ForInStatement(_)
                | AstNode::ForOfStatement(_)
                | AstNode::WhileStatement(_)
                | AstNode::DoWhileStatement(_)
                | AstNode::SwitchStatement(_)
                | AstNode::SwitchCase(_)
                | AstNode::WithStatement(_)
                | AstNode::TryStatement(_)
                | AstNode::CatchClause(_),
            ) => return false,

            // Everything else (export, label, exprs) -- keep walking.
            _ => {}
        }
        current = parent_id;
    }

    // If we exhaust ancestors without finding a container, treat as valid.
    true
}

impl LintRule for NoInnerDeclarations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-inner-declarations".to_owned(),
            description: "Disallow variable or function declarations in nested blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Function, AstNodeType::VariableDeclaration])
    }

    #[allow(clippy::match_same_arms)]
    fn run(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::Function(func) => {
                // Only flag function declarations (not expressions or arrow functions).
                // A function is a declaration if it has a name (id) -- named functions in blocks
                // are the concern. We also skip if `is_declare` (TypeScript ambient).
                if func.id.is_none() || func.is_declare {
                    return;
                }

                if !is_in_valid_position(node_id, ctx) {
                    let name = func.id.and_then(|id| {
                        if let Some(AstNode::BindingIdentifier(bi)) = ctx.node(id) {
                            Some(bi.name.clone())
                        } else {
                            None
                        }
                    });
                    let name_str = name.as_deref().unwrap_or("anonymous");
                    ctx.report(Diagnostic {
                        rule_name: "no-inner-declarations".to_owned(),
                        message: format!(
                            "Move function declaration '{name_str}' to program or function body root"
                        ),
                        span: Span::new(func.span.start, func.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::VariableDeclaration(decl) => {
                // Only flag `var` declarations (let/const are block-scoped by design).
                if decl.kind != VariableDeclarationKind::Var {
                    return;
                }

                if !is_in_valid_position(node_id, ctx) {
                    ctx.report(Diagnostic {
                        rule_name: "no-inner-declarations".to_owned(),
                        message: "Move variable declaration to program or function body root"
                            .to_owned(),
                        span: Span::new(decl.span.start, decl.span.end),
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInnerDeclarations)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_top_level_function() {
        let diags = lint("function foo() {}");
        assert!(diags.is_empty(), "top-level function should not be flagged");
    }

    #[test]
    fn test_allows_top_level_var() {
        let diags = lint("var x = 1;");
        assert!(diags.is_empty(), "top-level var should not be flagged");
    }

    #[test]
    fn test_allows_let_in_block() {
        let diags = lint("if (true) { let x = 1; }");
        assert!(diags.is_empty(), "let in block should not be flagged");
    }

    #[test]
    fn test_allows_const_in_block() {
        let diags = lint("if (true) { const x = 1; }");
        assert!(diags.is_empty(), "const in block should not be flagged");
    }

    #[test]
    fn test_flags_function_in_if() {
        let diags = lint("if (true) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in if-block should be flagged");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.rule_name == "no-inner-declarations"),
            "should come from no-inner-declarations"
        );
    }

    #[test]
    fn test_flags_var_in_if() {
        let diags = lint("if (true) { var x = 1; }");
        assert_eq!(diags.len(), 1, "var in if-block should be flagged");
    }

    #[test]
    fn test_flags_function_in_for() {
        let diags = lint("for (;;) { function foo() {} }");
        assert_eq!(diags.len(), 1, "function in for-loop should be flagged");
    }

    #[test]
    fn test_flags_var_in_while() {
        let diags = lint("while (true) { var x = 1; }");
        assert_eq!(diags.len(), 1, "var in while-loop should be flagged");
    }

    #[test]
    fn test_allows_function_in_function_body() {
        let diags = lint("function outer() { function inner() {} }");
        assert!(
            diags.is_empty(),
            "function in function body should not be flagged"
        );
    }

    #[test]
    fn test_allows_var_in_function_body() {
        let diags = lint("function outer() { var x = 1; }");
        assert!(
            diags.is_empty(),
            "var in function body should not be flagged"
        );
    }

    #[test]
    fn test_flags_function_in_nested_if_inside_function() {
        let diags = lint("function outer() { if (true) { function inner() {} } }");
        assert_eq!(
            diags.len(),
            1,
            "function in nested if inside function should be flagged"
        );
    }

    #[test]
    fn test_flags_var_in_try() {
        let diags = lint("try { var x = 1; } catch(e) {}");
        assert_eq!(diags.len(), 1, "var in try block should be flagged");
    }
}
