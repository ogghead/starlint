//! Rule: `func-names`
//!
//! Require or disallow named function expressions. Named functions
//! produce better stack traces and are easier to identify in debugging.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags anonymous function expressions that lack a name.
#[derive(Debug)]
pub struct FuncNames;

impl LintRule for FuncNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "func-names".to_owned(),
            description: "Require named function expressions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Function(func) = node else {
            return;
        };

        // Only check function expressions, not declarations.
        // Function declarations always have a name (parser enforces this),
        // so `id.is_none()` only matches function expressions.
        if func.id.is_none() {
            ctx.report(Diagnostic {
                rule_name: "func-names".to_owned(),
                message: "Unexpected unnamed function expression".to_owned(),
                span: Span::new(func.span.start, func.span.end),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(FuncNames)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_anonymous_function_expression() {
        let diags = lint("var foo = function() {};");
        assert_eq!(
            diags.len(),
            1,
            "anonymous function expression should be flagged"
        );
    }

    #[test]
    fn test_allows_named_function_expression() {
        let diags = lint("var foo = function bar() {};");
        assert!(
            diags.is_empty(),
            "named function expression should not be flagged"
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
    fn test_flags_anonymous_callback() {
        let diags = lint("arr.forEach(function() {});");
        assert_eq!(diags.len(), 1, "anonymous callback should be flagged");
    }

    #[test]
    fn test_allows_arrow_function() {
        // Arrow functions are inherently anonymous; this rule only targets `function` expressions
        let diags = lint("var foo = () => {};");
        assert!(
            diags.is_empty(),
            "arrow functions should not be flagged by func-names"
        );
    }
}
