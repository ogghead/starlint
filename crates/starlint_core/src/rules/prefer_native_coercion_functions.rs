//! Rule: `prefer-native-coercion-functions`
//!
//! Prefer passing native coercion functions like `Number`, `String`, or
//! `Boolean` directly instead of wrapping them in arrow functions.
//! `x => Number(x)` is equivalent to just `Number` and adds unnecessary
//! indirection.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Coercion function names that can be passed directly.
const COERCION_FUNCTIONS: &[&str] = &["Number", "String", "Boolean"];

/// Flags arrow functions that simply wrap a native coercion call.
#[derive(Debug)]
pub struct PreferNativeCoercionFunctions;

impl LintRule for PreferNativeCoercionFunctions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-native-coercion-functions".to_owned(),
            description:
                "Prefer passing `Number`, `String`, or `Boolean` directly instead of wrapping in an arrow function"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ArrowFunctionExpression(arrow) = node else {
            return;
        };

        // Must have exactly one parameter
        if arrow.params.len() != 1 {
            return;
        }

        // Must be an expression body (not a block body)
        if !arrow.expression {
            return;
        }

        // The parameter must be a simple binding identifier (not destructured)
        let Some(param_id) = arrow.params.first() else {
            return;
        };
        let Some(AstNode::BindingIdentifier(param_ident)) = ctx.node(*param_id) else {
            return;
        };
        let param_name = param_ident.name.clone();

        // The body is always a FunctionBody node. For expression arrows,
        // the FunctionBody contains one ExpressionStatement wrapping the expression.
        let Some(AstNode::FunctionBody(body)) = ctx.node(arrow.body) else {
            return;
        };

        if body.statements.len() != 1 {
            return;
        }

        let Some(stmt_id) = body.statements.first() else {
            return;
        };
        let Some(AstNode::ExpressionStatement(es)) = ctx.node(*stmt_id) else {
            return;
        };
        let expr_id = es.expression;

        // The expression must be a call to a coercion function
        let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
            return;
        };

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // Callee must be an identifier that is a coercion function
        let Some(AstNode::IdentifierReference(callee_id)) = ctx.node(call.callee) else {
            return;
        };
        let callee_name = callee_id.name.as_str();
        if !COERCION_FUNCTIONS.contains(&callee_name) {
            return;
        }

        // The single argument must be an identifier reference matching the parameter
        let Some(arg_id) = call.arguments.first() else {
            return;
        };
        let Some(AstNode::IdentifierReference(arg_ident)) = ctx.node(*arg_id) else {
            return;
        };

        if arg_ident.name.as_str() != param_name.as_str() {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "prefer-native-coercion-functions".to_owned(),
            message: format!("Unnecessary arrow function wrapper — pass `{callee_name}` directly"),
            span: Span::new(arrow.span.start, arrow.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace with `{callee_name}`")),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace with `{callee_name}`"),
                edits: vec![Edit {
                    span: Span::new(arrow.span.start, arrow.span.end),
                    replacement: callee_name.to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferNativeCoercionFunctions)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_number_wrapper() {
        let diags = lint("arr.map(x => Number(x));");
        assert_eq!(diags.len(), 1, "x => Number(x) should be flagged");
    }

    #[test]
    fn test_flags_string_wrapper() {
        let diags = lint("arr.map(x => String(x));");
        assert_eq!(diags.len(), 1, "x => String(x) should be flagged");
    }

    #[test]
    fn test_flags_boolean_wrapper() {
        let diags = lint("arr.map(x => Boolean(x));");
        assert_eq!(diags.len(), 1, "x => Boolean(x) should be flagged");
    }

    #[test]
    fn test_allows_parse_int_wrapper() {
        let diags = lint("arr.map(x => parseInt(x));");
        assert!(diags.is_empty(), "x => parseInt(x) should not be flagged");
    }

    #[test]
    fn test_allows_direct_coercion() {
        let diags = lint("arr.map(Number);");
        assert!(
            diags.is_empty(),
            "direct Number reference should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_param_name() {
        let diags = lint("arr.map(x => Number(y));");
        assert!(
            diags.is_empty(),
            "different argument name should not be flagged"
        );
    }

    #[test]
    fn test_allows_multiple_params() {
        let diags = lint("arr.map((x, i) => Number(x));");
        assert!(
            diags.is_empty(),
            "arrow with multiple params should not be flagged"
        );
    }

    #[test]
    fn test_allows_block_body() {
        let diags = lint("arr.map(x => { return Number(x); });");
        assert!(diags.is_empty(), "block body arrow should not be flagged");
    }

    #[test]
    fn test_allows_coercion_with_extra_args() {
        let diags = lint("arr.map(x => Number(x, 10));");
        assert!(
            diags.is_empty(),
            "coercion call with extra args should not be flagged"
        );
    }
}
