//! Rule: `typescript/no-unsafe-member-access`
//!
//! Disallow member access on `any` typed values. Accessing a property on
//! a value cast to `any` silently produces another `any`, allowing type
//! unsafety to cascade through the codebase without compiler warnings.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `(expr as any).property` patterns.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags static member access on expressions cast to `any`.
#[derive(Debug)]
pub struct NoUnsafeMemberAccess;

impl LintRule for NoUnsafeMemberAccess {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-member-access".to_owned(),
            description: "Disallow member access on `any` typed values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticMemberExpression(member) = node else {
            return;
        };

        if is_as_any_object(member.object, ctx) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-member-access".to_owned(),
                message: "Unsafe member access — accessing a property on an `as any` expression propagates type unsafety".to_owned(),
                span: Span::new(member.span.start, member.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check whether an object expression is an `as any` cast.
/// Uses source text heuristic. No `ParenthesizedExpression` exists in
/// `starlint_ast` (parens are transparent in the AST).
fn is_as_any_object(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeMemberAccess)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_member_access_on_as_any() {
        let diags = lint("let x = (value as any).foo;");
        assert_eq!(diags.len(), 1, "`(value as any).foo` should be flagged");
    }

    #[test]
    fn test_flags_nested_member_access() {
        let diags = lint("let x = (getData() as any).result;");
        assert_eq!(
            diags.len(),
            1,
            "`(getData() as any).result` should be flagged"
        );
    }

    #[test]
    fn test_allows_typed_member_access() {
        let diags = lint("let x = (value as Record<string, number>).foo;");
        assert!(
            diags.is_empty(),
            "member access on typed assertion should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_member_access() {
        let diags = lint("let x = obj.foo;");
        assert!(
            diags.is_empty(),
            "plain member access should not be flagged"
        );
    }

    #[test]
    fn test_flags_parenthesized_as_any_member() {
        let diags = lint("let x = ((value as any)).bar;");
        assert_eq!(
            diags.len(),
            1,
            "double-parenthesized `as any` member access should be flagged"
        );
    }
}
