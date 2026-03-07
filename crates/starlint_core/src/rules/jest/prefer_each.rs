//! Rule: `jest/prefer-each`
//!
//! Suggest `test.each` over repeated similar tests. When a `describe` block
//! contains 3 or more `it`/`test` calls with titles sharing a common prefix,
//! the test suite would benefit from parameterization via `test.each`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `describe` blocks with 3+ similarly-titled test cases.
#[derive(Debug)]
pub struct PreferEach;

/// Minimum number of tests with the same prefix to trigger the rule.
const MIN_SIMILAR_TESTS: usize = 3;

/// Minimum prefix length (in chars) to consider titles "similar".
const MIN_PREFIX_LEN: usize = 5;

impl LintRule for PreferEach {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-each".to_owned(),
            description: "Suggest using `test.each` over repeated similar tests".to_owned(),
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

        // Must be `describe(...)` call
        let is_describe = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "describe"
        );
        if !is_describe {
            return;
        }

        // Second argument should be the callback
        let Some(second_arg_id) = call.arguments.get(1) else {
            return;
        };

        // Get the function body NodeId
        let body_id = match ctx.node(*second_arg_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => arrow.body,
            Some(AstNode::Function(func)) => {
                let Some(body) = func.body else {
                    return;
                };
                body
            }
            _ => return,
        };

        // Get the body node (BlockStatement or FunctionBody)
        let body_stmts: Box<[NodeId]> = match ctx.node(body_id) {
            Some(AstNode::FunctionBody(fb)) => fb.statements.clone(),
            Some(AstNode::BlockStatement(block)) => block.body.clone(),
            _ => return,
        };

        // Collect test titles from top-level `it`/`test` calls in the body
        let mut titles: Vec<String> = Vec::new();
        for stmt_id in &body_stmts {
            let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(*stmt_id) else {
                continue;
            };
            let Some(AstNode::CallExpression(inner_call)) = ctx.node(expr_stmt.expression) else {
                continue;
            };
            let is_test = matches!(
                ctx.node(inner_call.callee),
                Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "it" || id.name.as_str() == "test"
            );
            if !is_test {
                continue;
            }
            if let Some(first_arg_id) = inner_call.arguments.first() {
                if let Some(AstNode::StringLiteral(s)) = ctx.node(*first_arg_id) {
                    titles.push(s.value.clone());
                }
            }
        }

        if titles.len() < MIN_SIMILAR_TESTS {
            return;
        }

        // Check if titles share a common prefix of meaningful length
        if let Some(first) = titles.first() {
            let prefix_len = titles.iter().skip(1).fold(first.len(), |acc, title| {
                let common = first
                    .chars()
                    .zip(title.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
                if common < acc { common } else { acc }
            });

            if prefix_len >= MIN_PREFIX_LEN {
                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-each".to_owned(),
                    message: "Consider using `test.each` to parameterize these similar test cases"
                        .to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferEach)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_similar_tests() {
        let source = r"
describe('math', () => {
    test('handles addition with 1', () => {});
    test('handles addition with 2', () => {});
    test('handles addition with 3', () => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "3+ tests with similar titles should be flagged"
        );
    }

    #[test]
    fn test_allows_different_titles() {
        let source = r"
describe('math', () => {
    test('adds numbers', () => {});
    test('subtracts numbers', () => {});
    test('multiplies numbers', () => {});
});
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "tests with different titles should not be flagged"
        );
    }

    #[test]
    fn test_allows_few_tests() {
        let source = r"
describe('math', () => {
    test('handles input 1', () => {});
    test('handles input 2', () => {});
});
";
        let diags = lint(source);
        assert!(diags.is_empty(), "fewer than 3 tests should not be flagged");
    }
}
