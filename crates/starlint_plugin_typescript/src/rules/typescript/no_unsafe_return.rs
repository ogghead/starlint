//! Rule: `typescript/no-unsafe-return`
//!
//! Disallow returning `any` typed values from functions. Returning a value
//! cast to `any` defeats the purpose of the function's return type annotation,
//! allowing callers to receive an untyped value without any compiler warning.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `return expr as any` patterns.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags return statements whose argument is an `as any` assertion.
#[derive(Debug)]
pub struct NoUnsafeReturn;

impl LintRule for NoUnsafeReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-return".to_owned(),
            description: "Disallow returning `any` typed values from functions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ReturnStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ReturnStatement(ret) = node else {
            return;
        };

        let Some(arg_id) = ret.argument else {
            return;
        };

        if is_as_any_return(arg_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-return".to_owned(),
                message: "Unsafe return — returning an `as any` value defeats the function's return type safety".to_owned(),
                span: Span::new(ret.span.start, ret.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check whether a return argument is an `as any` cast.
/// Uses source text heuristic. No `ParenthesizedExpression` exists in
/// `starlint_ast` (parens are transparent in the AST).
fn is_as_any_return(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::TSAsExpression(as_expr)) = ctx.node(node_id) else {
        return false;
    };
    let source = ctx.source_text();
    let start = usize::try_from(as_expr.span.start).unwrap_or(0);
    let end = usize::try_from(as_expr.span.end).unwrap_or(0);
    source
        .get(start..end)
        .is_some_and(|text| text.trim_end().ends_with("as any"))
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnsafeReturn, "test.ts");

    #[test]
    fn test_flags_return_as_any() {
        let diags = lint("function f() { return value as any; }");
        assert_eq!(diags.len(), 1, "`return value as any` should be flagged");
    }

    #[test]
    fn test_flags_parenthesized_return_as_any() {
        let diags = lint("function f() { return (value as any); }");
        assert_eq!(
            diags.len(),
            1,
            "parenthesized `return (value as any)` should be flagged"
        );
    }

    #[test]
    fn test_allows_return_as_string() {
        let diags = lint("function f() { return value as string; }");
        assert!(
            diags.is_empty(),
            "`return value as string` should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_return() {
        let diags = lint("function f() { return 42; }");
        assert!(diags.is_empty(), "plain return should not be flagged");
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("function f() { return; }");
        assert!(diags.is_empty(), "empty return should not be flagged");
    }
}
