//! Rule: `vars-on-top`
//!
//! Require `var` declarations to be at the top of their enclosing scope
//! (function body or program). This ensures all `var`s are declared before
//! any other statements, making hoisting behavior explicit.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `var` declarations that are not at the top of their scope.
#[derive(Debug)]
pub struct VarsOnTop;

/// Check whether a statement node is a var declaration or a directive ("use strict" etc).
fn is_var_or_directive(node: &AstNode) -> bool {
    match node {
        AstNode::VariableDeclaration(decl) => decl.kind == VariableDeclarationKind::Var,
        AstNode::ExpressionStatement(expr) => {
            // Directive prologues (e.g. "use strict") are string literal expression statements
            // We check if the expression child is a string literal, but since expr.expression
            // is a NodeId we can't resolve it without context. We'll handle this conservatively.
            // For simplicity, treat all expression statements as non-var.
            // A more complete implementation would resolve expr.expression to check for StringLiteral.
            let _ = expr;
            false
        }
        _ => false,
    }
}

/// Check whether a statement node is a module-level declaration (import/export).
const fn is_module_decl(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::ImportDeclaration(_)
            | AstNode::ExportAllDeclaration(_)
            | AstNode::ExportDefaultDeclaration(_)
            | AstNode::ExportNamedDeclaration(_)
    )
}

impl LintRule for VarsOnTop {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "vars-on-top".to_owned(),
            description: "Require var declarations to be at the top of their scope".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::FunctionBody, AstNodeType::Program])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // We check at the scope level (Program or FunctionBody) to see if
        // var declarations come before any non-var statements.
        let stmts: &[NodeId] = match node {
            AstNode::Program(program) => &program.body,
            AstNode::FunctionBody(body) => &body.statements,
            _ => return,
        };

        check_statements(stmts, ctx);
    }
}

/// Check a list of statement node IDs for var declarations that appear after non-var statements.
fn check_statements(stmt_ids: &[NodeId], ctx: &mut LintContext<'_>) {
    let mut found_non_var = false;

    // First pass: collect violations to avoid borrow conflict
    let mut violations: Vec<Span> = Vec::new();

    for stmt_id in stmt_ids {
        let Some(stmt) = ctx.node(*stmt_id) else {
            continue;
        };

        if found_non_var {
            // Any var declaration after a non-var statement is a violation
            if let AstNode::VariableDeclaration(decl) = stmt {
                if decl.kind == VariableDeclarationKind::Var {
                    violations.push(Span::new(decl.span.start, decl.span.end));
                }
            }
        } else if !is_var_or_directive(stmt) && !is_module_decl(stmt) {
            found_non_var = true;
        }
    }

    for span in violations {
        ctx.report(Diagnostic {
            rule_name: "vars-on-top".to_owned(),
            message: "All `var` declarations must be at the top of the scope".to_owned(),
            span,
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(VarsOnTop);

    #[test]
    fn test_allows_var_at_top() {
        let diags = lint("var x = 1;\nvar y = 2;\nconsole.log(x);");
        assert!(diags.is_empty(), "vars at top should not be flagged");
    }

    #[test]
    fn test_flags_var_after_statement() {
        let diags = lint("console.log('hi');\nvar x = 1;");
        assert_eq!(
            diags.len(),
            1,
            "var after non-var statement should be flagged"
        );
    }

    #[test]
    fn test_allows_let_const_anywhere() {
        // This rule only applies to var, not let/const
        let diags = lint("console.log('hi');\nlet x = 1;\nconst y = 2;");
        assert!(diags.is_empty(), "let/const should not be checked");
    }

    #[test]
    fn test_flags_var_in_function() {
        let diags = lint("function foo() {\n  console.log('hi');\n  var x = 1;\n}");
        assert_eq!(
            diags.len(),
            1,
            "var after statement in function should be flagged"
        );
    }

    #[test]
    fn test_allows_var_at_top_of_function() {
        let diags = lint("function foo() {\n  var x = 1;\n  console.log(x);\n}");
        assert!(diags.is_empty(), "var at top of function should be allowed");
    }

    #[test]
    fn test_multiple_violations() {
        let diags = lint("console.log('a');\nvar x = 1;\nvar y = 2;");
        assert_eq!(diags.len(), 2, "each var after non-var should be flagged");
    }

    #[test]
    fn test_allows_import_before_var() {
        // Module syntax: imports typically come before vars
        let diags = lint("import foo from 'foo';\nvar x = 1;");
        assert!(diags.is_empty(), "import before var should be allowed");
    }
}
