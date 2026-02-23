//! Rule: `typescript/explicit-function-return-type`
//!
//! Require explicit return types on functions and class methods. Functions
//! without explicit return types rely on type inference which can be fragile
//! and may lead to unexpected API contracts. Arrow functions that are
//! immediately assigned (e.g. callbacks) are excluded as this is a common
//! and acceptable pattern.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/explicit-function-return-type";

/// Flags functions and class methods that lack an explicit return type annotation.
#[derive(Debug)]
pub struct ExplicitFunctionReturnType;

impl LintRule for ExplicitFunctionReturnType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require explicit return types on functions and class methods".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Function])
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Function(func) = node else {
            return;
        };

        // Only check function declarations and expressions that have a body
        // (skip ambient declarations like `declare function ...`)
        let Some(body_id) = func.body else {
            return;
        };

        // FunctionNode doesn't have a `return_type` field in starlint_ast.
        // Use source text heuristic: check for `:` between the closing `)` of
        // params and the opening `{` of the body.
        let Some(body_node) = ctx.node(body_id) else {
            return;
        };
        let body_start = body_node.span().start;
        let source = ctx.source_text();
        // Find the last `)` before the body
        let region_end = usize::try_from(body_start).unwrap_or(0);
        let func_start = usize::try_from(func.span.start).unwrap_or(0);
        let between = source.get(func_start..region_end).unwrap_or("");
        // Look for `)` followed (after optional whitespace) by `:`
        if let Some(paren_pos) = between.rfind(')') {
            let after_paren = between.get(paren_pos + 1..).unwrap_or("").trim_start();
            if after_paren.starts_with(':') {
                // Has a return type annotation
                return;
            }
        }

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: "Missing return type on function — add an explicit return type annotation"
                .to_owned(),
            span: Span::new(func.span.start, func.span.end),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExplicitFunctionReturnType)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_function_without_return_type() {
        let diags = lint("function foo() { return 1; }");
        assert_eq!(
            diags.len(),
            1,
            "function without return type should be flagged"
        );
    }

    #[test]
    fn test_allows_function_with_return_type() {
        let diags = lint("function foo(): number { return 1; }");
        assert!(
            diags.is_empty(),
            "function with return type should not be flagged"
        );
    }

    #[test]
    fn test_flags_method_without_return_type() {
        let diags = lint("class Foo { bar() { return 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "class method without return type should be flagged"
        );
    }

    #[test]
    fn test_allows_method_with_return_type() {
        let diags = lint("class Foo { bar(): number { return 1; } }");
        assert!(
            diags.is_empty(),
            "class method with return type should not be flagged"
        );
    }

    #[test]
    fn test_allows_declare_function() {
        let diags = lint("declare function foo(): void;");
        assert!(
            diags.is_empty(),
            "declare function should not be flagged (no body)"
        );
    }
}
