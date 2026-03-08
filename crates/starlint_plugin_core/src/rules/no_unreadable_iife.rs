//! Rule: `no-unreadable-iife`
//!
//! Disallow unreadable IIFEs (Immediately Invoked Function Expressions).
//! Traditional IIFEs using `(function() { ... })()` are harder to read than
//! arrow function IIFEs. Arrow IIFEs like `(() => ...)()` are excluded because
//! they are considered more readable.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags non-arrow IIFEs.
#[derive(Debug)]
pub struct NoUnreadableIife;

impl LintRule for NoUnreadableIife {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unreadable-iife".to_owned(),
            description: "Disallow unreadable IIFEs (non-arrow function IIFEs)".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check if the callee is a function expression (possibly wrapped in parens).
        // This covers `(function() {})()` — the outer call with a parenthesized
        // function expression as callee.
        if is_function_iife_callee(call.callee, ctx) {
            ctx.report(Diagnostic {
                rule_name: "no-unreadable-iife".to_owned(),
                message: "IIFE with `function` expression is hard to read; consider using an arrow function".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a non-arrow function expression, unwrapping parens.
///
/// This handles:
/// - `(function() { ... })()` — callee is `ParenthesizedExpression(FunctionExpression)`
/// - `(function() { ... }())` — in this form, the inner call's callee is the
///   `FunctionExpression` directly, and the outer paren wraps the `CallExpression`.
///   However, in oxc's AST the `(function() {}())` form also parses as a
///   `CallExpression` whose callee is a `FunctionExpression` inside parens.
fn is_function_iife_callee(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::Function(_)) => true,
        // Note: starlint_ast does not have ParenthesizedExpression; the parser
        // unwraps parentheses, so the callee points directly to the function.
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnreadableIife)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_function_iife_outer_parens() {
        let diags = lint("(function() { return 1; })();");
        assert_eq!(
            diags.len(),
            1,
            "function IIFE with outer parens should be flagged"
        );
    }

    #[test]
    fn test_flags_function_iife_inner_parens() {
        let diags = lint("(function() { return 1; }());");
        assert_eq!(
            diags.len(),
            1,
            "function IIFE with inner parens should be flagged"
        );
    }

    #[test]
    fn test_allows_arrow_iife() {
        let diags = lint("(() => 1)();");
        assert!(diags.is_empty(), "arrow IIFE should not be flagged");
    }

    #[test]
    fn test_allows_normal_function_declaration() {
        let diags = lint("function foo() { return 1; }");
        assert!(
            diags.is_empty(),
            "function declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_function_call() {
        let diags = lint("foo();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }

    #[test]
    fn test_flags_named_function_iife() {
        let diags = lint("(function myFunc() { return 1; })();");
        assert_eq!(diags.len(), 1, "named function IIFE should be flagged");
    }
}
