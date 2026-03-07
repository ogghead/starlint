//! Rule: `func-style`
//!
//! Enforce consistent function style. By default, prefers function declarations
//! over `const` function expressions. Arrow functions are allowed.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `const foo = function() {}` — prefer function declarations.
#[derive(Debug)]
pub struct FuncStyle;

impl LintRule for FuncStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "func-style".to_owned(),
            description: "Enforce consistent use of function declarations vs expressions"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclarator])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclarator(decl) = node else {
            return;
        };

        let Some(init_id) = decl.init else {
            return;
        };

        // Only flag function expressions, not arrow functions
        if matches!(ctx.node(init_id), Some(AstNode::Function(_))) {
            ctx.report(Diagnostic {
                rule_name: "func-style".to_owned(),
                message: "Use a function declaration instead of a const function expression"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(FuncStyle)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_const_function_expression() {
        let diags = lint("const foo = function() {};");
        assert_eq!(
            diags.len(),
            1,
            "const function expression should be flagged"
        );
    }

    #[test]
    fn test_flags_named_function_expression() {
        let diags = lint("const foo = function bar() {};");
        assert_eq!(
            diags.len(),
            1,
            "named const function expression should be flagged"
        );
    }

    #[test]
    fn test_allows_function_declaration() {
        let diags = lint("function foo() {}");
        assert!(
            diags.is_empty(),
            "function declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_arrow_function() {
        let diags = lint("const foo = () => {};");
        assert!(diags.is_empty(), "arrow function should not be flagged");
    }

    #[test]
    fn test_allows_non_function_init() {
        let diags = lint("const foo = 42;");
        assert!(diags.is_empty(), "numeric assignment should not be flagged");
    }
}
