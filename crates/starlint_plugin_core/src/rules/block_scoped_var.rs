//! Rule: `block-scoped-var`
//!
//! Flag `var` declarations inside blocks where developers likely expect
//! block-scoping (like `let`). Because `var` is function-scoped, a `var`
//! inside an `if`, `for`, `while`, `try`, or similar control-flow block
//! can lead to confusing hoisting bugs. Suggest using `let` or `const`
//! instead.
//!
//! Uses a stack-based approach: tracks function and control-flow boundaries
//! during traversal. When a `var` is encountered while the nearest scope
//! boundary is a control-flow statement (not a function), it is flagged.

use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Marker for the kind of scope boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    /// A function boundary (resets block context).
    Function,
    /// A control-flow statement that introduces a block scope.
    Block,
}

/// Flags `var` declarations inside control-flow blocks.
#[derive(Debug)]
pub struct BlockScopedVar {
    /// Stack tracking scope boundaries during traversal.
    scopes: RwLock<Vec<ScopeKind>>,
}

impl BlockScopedVar {
    /// Create a new `BlockScopedVar` rule instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scopes: RwLock::new(Vec::new()),
        }
    }
}

impl Default for BlockScopedVar {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if an `AstNode` is a function boundary.
const fn is_function_boundary(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::Function(_) | AstNode::ArrowFunctionExpression(_)
    )
}

/// Check if an `AstNode` is a control-flow statement that introduces a
/// block scope (where a `var` would be surprising).
const fn is_block_scope(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::IfStatement(_)
            | AstNode::ForStatement(_)
            | AstNode::ForInStatement(_)
            | AstNode::ForOfStatement(_)
            | AstNode::WhileStatement(_)
            | AstNode::DoWhileStatement(_)
            | AstNode::TryStatement(_)
            | AstNode::SwitchStatement(_)
            | AstNode::LabeledStatement(_)
    )
}

impl LintRule for BlockScopedVar {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "block-scoped-var".to_owned(),
            description:
                "Flag `var` declarations inside blocks where `let`/`const` is likely intended"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ArrowFunctionExpression,
            AstNodeType::DoWhileStatement,
            AstNodeType::ForInStatement,
            AstNodeType::ForOfStatement,
            AstNodeType::ForStatement,
            AstNodeType::Function,
            AstNodeType::IfStatement,
            AstNodeType::LabeledStatement,
            AstNodeType::SwitchStatement,
            AstNodeType::TryStatement,
            AstNodeType::VariableDeclaration,
            AstNodeType::WhileStatement,
        ])
    }

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ArrowFunctionExpression,
            AstNodeType::DoWhileStatement,
            AstNodeType::ForInStatement,
            AstNodeType::ForOfStatement,
            AstNodeType::ForStatement,
            AstNodeType::Function,
            AstNodeType::IfStatement,
            AstNodeType::LabeledStatement,
            AstNodeType::SwitchStatement,
            AstNodeType::TryStatement,
            AstNodeType::VariableDeclaration,
            AstNodeType::WhileStatement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Push scope markers for function boundaries.
        if is_function_boundary(node) {
            if let Ok(mut stack) = self.scopes.write() {
                stack.push(ScopeKind::Function);
            }
            return;
        }

        // Push scope markers for control-flow statements.
        if is_block_scope(node) {
            if let Ok(mut stack) = self.scopes.write() {
                stack.push(ScopeKind::Block);
            }
            return;
        }

        // Check var declarations.
        let AstNode::VariableDeclaration(decl) = node else {
            return;
        };

        if decl.kind != VariableDeclarationKind::Var {
            return;
        }

        let Ok(stack) = self.scopes.read() else {
            return;
        };

        // If the most recent scope boundary is a Block (not a Function),
        // the var is inside a control-flow block.
        let in_block = stack.last().is_some_and(|scope| *scope == ScopeKind::Block);

        if in_block {
            ctx.report(Diagnostic {
                rule_name: "block-scoped-var".to_owned(),
                message: "`var` declaration inside a block — consider using `let` or `const` for block scoping".to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }

    fn leave(&self, _node_id: NodeId, node: &AstNode, _ctx: &mut LintContext<'_>) {
        if is_function_boundary(node) || is_block_scope(node) {
            if let Ok(mut stack) = self.scopes.write() {
                let _ = stack.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(BlockScopedVar::new());

    #[test]
    fn test_flags_var_in_if_block() {
        let diags = lint("if (true) { var x = 1; }");
        assert_eq!(diags.len(), 1, "var in if-block should be flagged");
    }

    #[test]
    fn test_allows_function_scope_var() {
        let diags = lint("function foo() { var x = 1; }");
        assert!(
            diags.is_empty(),
            "var in function body should not be flagged"
        );
    }

    #[test]
    fn test_allows_program_scope_var() {
        let diags = lint("var x = 1;");
        assert!(
            diags.is_empty(),
            "var at program scope should not be flagged"
        );
    }

    #[test]
    fn test_flags_var_in_for_statement() {
        // `for (var i = 0; ...)` — the ForStatement itself pushes Block scope,
        // so var declarations inside it (including for-init) are flagged.
        let diags = lint("for (var i = 0; i < 10; i++) { use(i); }");
        assert_eq!(diags.len(), 1, "var inside for statement should be flagged");
    }

    #[test]
    fn test_flags_var_in_while_block() {
        let diags = lint("while (true) { var x = 1; }");
        assert_eq!(diags.len(), 1, "var in while-block should be flagged");
    }

    #[test]
    fn test_allows_let_in_block() {
        let diags = lint("if (true) { let x = 1; }");
        assert!(
            diags.is_empty(),
            "let in block should not be flagged (only var is checked)"
        );
    }

    #[test]
    fn test_flags_var_in_try_block() {
        let diags = lint("try { var x = 1; } catch (e) {}");
        assert_eq!(diags.len(), 1, "var in try-block should be flagged");
    }

    #[test]
    fn test_allows_var_in_nested_function() {
        let diags = lint("if (true) { function foo() { var x = 1; } }");
        assert!(
            diags.is_empty(),
            "var in nested function body should not be flagged"
        );
    }
}
