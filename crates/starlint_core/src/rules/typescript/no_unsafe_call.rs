//! Rule: `typescript/no-unsafe-call`
//!
//! Disallow calling `any` typed values. Calling a value cast to `any`
//! bypasses all parameter and return type checking, allowing runtime
//! type errors to go undetected at compile time.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `(expr as any)(...)` patterns.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags call expressions where the callee is cast to `any`.
#[derive(Debug)]
pub struct NoUnsafeCall;

impl LintRule for NoUnsafeCall {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-call".to_owned(),
            description: "Disallow calling `any` typed values".to_owned(),
            category: Category::Correctness,
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

        if is_as_any_callee(call.callee, ctx) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-call".to_owned(),
                message: "Unsafe call — calling an `as any` expression bypasses argument and return type checking".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check whether a callee expression is an `as any` cast.
/// Uses source text heuristic. No `ParenthesizedExpression` exists in
/// `starlint_ast` (parens are transparent in the AST).
fn is_as_any_callee(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeCall)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_call_as_any() {
        let diags = lint("(getValue as any)();");
        assert_eq!(diags.len(), 1, "`(getValue as any)()` should be flagged");
    }

    #[test]
    fn test_flags_nested_paren_call_as_any() {
        let diags = lint("((fn as any))();");
        assert_eq!(
            diags.len(),
            1,
            "double-parenthesized `as any` call should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("getValue();");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }

    #[test]
    fn test_allows_call_as_string() {
        let diags = lint("(getValue as Function)();");
        assert!(
            diags.is_empty(),
            "`as Function` call should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_typed_call() {
        let diags = lint("(getValue as () => number)();");
        assert!(
            diags.is_empty(),
            "typed function call should not be flagged"
        );
    }
}
