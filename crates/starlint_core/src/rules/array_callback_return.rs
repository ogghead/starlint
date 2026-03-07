//! Rule: `array-callback-return`
//!
//! Enforce `return` statements in callbacks of array methods. Methods like
//! `map`, `filter`, `reduce`, `find`, `every`, `some`, `sort`, `flatMap`,
//! and `findIndex` expect their callbacks to return a value. Forgetting
//! to return is a common bug.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Array methods whose callbacks must return a value.
const METHODS_REQUIRING_RETURN: &[&str] = &[
    "map",
    "filter",
    "reduce",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "every",
    "some",
    "sort",
    "flatMap",
];

/// Flags callbacks in array methods that don't return a value.
#[derive(Debug)]
pub struct ArrayCallbackReturn;

impl LintRule for ArrayCallbackReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "array-callback-return".to_owned(),
            description: "Enforce `return` in callbacks of array methods".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check if callee is `something.map(...)`, `something.filter(...)`, etc.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();
        if !METHODS_REQUIRING_RETURN.contains(&method_name) {
            return;
        }

        // Check the first argument (the callback)
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        let Some(first_arg) = ctx.node(*first_arg_id) else {
            return;
        };

        match first_arg {
            AstNode::Function(func) => {
                if let Some(body_id) = func.body {
                    if let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) {
                        if !statements_contain_return(&body.statements, ctx) {
                            ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                                rule_name: "array-callback-return".to_owned(),
                                message: format!(
                                    "Expected a return value in `.{method_name}()` callback"
                                ),
                                span: Span::new(func.span.start, func.span.end),
                                severity: Severity::Error,
                                help: Some(format!(
                                    "Array `.{method_name}()` expects a return value from its callback"
                                )),
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                }
            }
            AstNode::ArrowFunctionExpression(arrow) => {
                // Arrow functions with expression bodies always return
                if arrow.expression {
                    return;
                }
                if let Some(AstNode::FunctionBody(body)) = ctx.node(arrow.body) {
                    if !statements_contain_return(&body.statements, ctx) {
                        ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                            rule_name: "array-callback-return".to_owned(),
                            message: format!(
                                "Expected a return value in `.{method_name}()` callback"
                            ),
                            span: Span::new(arrow.span.start, arrow.span.end),
                            severity: Severity::Error,
                            help: Some(format!(
                                "Array `.{method_name}()` expects a return value from its callback"
                            )),
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if statements contain a return statement with a value.
fn statements_contain_return(stmts: &[NodeId], ctx: &LintContext<'_>) -> bool {
    stmts.iter().any(|s| statement_contains_return(*s, ctx))
}

/// Recursively check a statement for a return with a value.
fn statement_contains_return(stmt_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(stmt) = ctx.node(stmt_id) else {
        return false;
    };
    match stmt {
        AstNode::ReturnStatement(ret) => ret.argument.is_some(),
        AstNode::BlockStatement(block) => statements_contain_return(&block.body, ctx),
        AstNode::IfStatement(if_stmt) => {
            statement_contains_return(if_stmt.consequent, ctx)
                || if_stmt
                    .alternate
                    .is_some_and(|alt| statement_contains_return(alt, ctx))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ArrayCallbackReturn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_map_without_return() {
        let diags = lint("[1,2,3].map(function(x) { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            "map callback without return should be flagged"
        );
    }

    #[test]
    fn test_allows_map_with_return() {
        let diags = lint("[1,2,3].map(function(x) { return x * 2; });");
        assert!(
            diags.is_empty(),
            "map callback with return should not be flagged"
        );
    }

    #[test]
    fn test_allows_arrow_expression() {
        let diags = lint("[1,2,3].map(x => x * 2);");
        assert!(
            diags.is_empty(),
            "arrow expression callback should not be flagged"
        );
    }

    #[test]
    fn test_flags_filter_without_return() {
        let diags = lint("[1,2,3].filter(function(x) { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            "filter callback without return should be flagged"
        );
    }

    #[test]
    fn test_allows_for_each() {
        let diags = lint("[1,2,3].forEach(function(x) { console.log(x); });");
        assert!(diags.is_empty(), "forEach callback should not be flagged");
    }

    #[test]
    fn test_flags_arrow_block_without_return() {
        let diags = lint("[1,2,3].map(x => { console.log(x); });");
        assert_eq!(
            diags.len(),
            1,
            "arrow block callback without return should be flagged"
        );
    }
}
