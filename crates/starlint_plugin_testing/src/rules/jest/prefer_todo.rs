//! Rule: `jest/prefer-todo`
//!
//! Suggest `test.todo('title')` for empty test cases. An empty test body
//! (or one with no assertions) is likely a placeholder; using `test.todo`
//! makes the intent explicit and appears in test runner summaries.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags empty `it()` / `test()` callbacks that should use `test.todo()`.
#[derive(Debug)]
pub struct PreferTodo;

impl LintRule for PreferTodo {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-todo".to_owned(),
            description: "Suggest using `test.todo()` for empty test cases".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `it(...)` or `test(...)`
        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.clone(),
            _ => return,
        };
        if callee_name.as_str() != "it" && callee_name.as_str() != "test" {
            return;
        }

        // Must have at least 2 arguments (title, callback)
        if call.arguments.len() < 2 {
            return;
        }
        let Some(&second_arg_id) = call.arguments.get(1) else {
            return;
        };

        let Some(callback_expr) = ctx.node(second_arg_id) else {
            return;
        };

        // Check if the callback has an empty body
        let is_empty = match callback_expr {
            AstNode::ArrowFunctionExpression(arrow) => is_body_empty(arrow.body, ctx),
            AstNode::Function(func) => func.body.is_some_and(|b| is_body_empty(b, ctx)),
            _ => false,
        };

        if is_empty {
            let source = ctx.source_text();
            #[allow(clippy::as_conversions)]
            let fix = call.arguments.first().and_then(|&a| {
                let sp = ctx.node(a)?.span();
                let title = source.get(sp.start as usize..sp.end as usize)?.to_owned();
                let replacement = format!("{callee_name}.todo({title})");
                Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                })
            });

            ctx.report(Diagnostic {
                rule_name: "jest/prefer-todo".to_owned(),
                message: "Use `test.todo()` instead of an empty test callback".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `test.todo()`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if a function body node is empty (no statements, or only empty statements).
fn is_body_empty(body_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
        return false;
    };
    if body.statements.is_empty() {
        return true;
    }
    // Also treat bodies with only empty statements as empty
    body.statements
        .iter()
        .all(|&s| matches!(ctx.node(s), Some(AstNode::EmptyStatement(_))))
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferTodo)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_arrow_test() {
        let diags = lint("test('should work', () => {});");
        assert_eq!(diags.len(), 1, "empty arrow test should be flagged");
    }

    #[test]
    fn test_flags_empty_function_test() {
        let diags = lint("it('should work', function() {});");
        assert_eq!(diags.len(), 1, "empty function test should be flagged");
    }

    #[test]
    fn test_allows_test_with_body() {
        let diags = lint("test('should work', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "test with body should not be flagged");
    }
}
