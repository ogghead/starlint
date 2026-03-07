//! Rule: `typescript/no-unsafe-assignment`
//!
//! Disallow assigning `any` typed values. Assigning a value cast to `any`
//! silently removes type safety for the receiving binding, allowing type
//! errors to propagate undetected through the codebase.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `const x = expr as any` patterns.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags variable declarations initialized with an `as any` assertion.
#[derive(Debug)]
pub struct NoUnsafeAssignment;

impl LintRule for NoUnsafeAssignment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-assignment".to_owned(),
            description: "Disallow assigning `any` typed values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclarator])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclarator(decl) = node else {
            return;
        };

        let Some(init_id) = decl.init else {
            return;
        };

        if is_as_any(init_id, ctx) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-assignment".to_owned(),
                message: "Unsafe assignment — assigning an `as any` value removes type safety for this binding".to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check whether a node is a `TSAsExpression` casting to `any`.
/// Uses source text heuristic since `TSAsExpressionNode` has no
/// `type_annotation` field. No `ParenthesizedExpression` exists in
/// `starlint_ast` (parens are transparent in the AST).
fn is_as_any(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafeAssignment)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_const_as_any() {
        let diags = lint("const x = value as any;");
        assert_eq!(diags.len(), 1, "`const x = value as any` should be flagged");
    }

    #[test]
    fn test_flags_let_as_any() {
        let diags = lint("let y = getData() as any;");
        assert_eq!(
            diags.len(),
            1,
            "`let y = getData() as any` should be flagged"
        );
    }

    #[test]
    fn test_flags_parenthesized_as_any() {
        let diags = lint("const z = (value as any);");
        assert_eq!(
            diags.len(),
            1,
            "parenthesized `as any` assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_as_string() {
        let diags = lint("const x = value as string;");
        assert!(diags.is_empty(), "`as string` should not be flagged");
    }

    #[test]
    fn test_allows_plain_assignment() {
        let diags = lint("const x = 42;");
        assert!(diags.is_empty(), "plain assignment should not be flagged");
    }
}
