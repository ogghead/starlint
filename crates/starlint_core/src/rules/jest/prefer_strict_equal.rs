//! Rule: `jest/prefer-strict-equal`
//!
//! Suggest `toStrictEqual` over `toEqual`. `toStrictEqual` checks that
//! objects have the same type and structure, unlike `toEqual` which performs
//! a more lenient recursive comparison that ignores `undefined` properties.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.toEqual()` calls that could use `.toStrictEqual()`.
#[derive(Debug)]
pub struct PreferStrictEqual;

impl LintRule for PreferStrictEqual {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-strict-equal".to_owned(),
            description: "Suggest using `toStrictEqual()` over `toEqual()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        if member.property.as_str() != "toEqual" {
            return;
        }

        if !is_expect_chain(member.object, ctx) {
            return;
        }

        // Property is a String, compute span from source text
        let source = ctx.source_text();
        let member_start = usize::try_from(member.span.start).unwrap_or(0);
        let member_end = usize::try_from(member.span.end).unwrap_or(0);
        let member_text = source.get(member_start..member_end).unwrap_or("");
        let prop_offset = member_text.rfind('.').map_or(0, |i| i + 1);
        #[allow(clippy::as_conversions)]
        let prop_start = member.span.start + prop_offset as u32;
        #[allow(clippy::as_conversions)]
        let prop_end = prop_start + "toEqual".len() as u32;
        let prop_span = Span::new(prop_start, prop_end);

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-strict-equal".to_owned(),
            message: "Use `toStrictEqual()` instead of `toEqual()` for stricter equality checking"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace `toEqual` with `toStrictEqual`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `toStrictEqual`".to_owned(),
                edits: vec![Edit {
                    span: prop_span,
                    replacement: "toStrictEqual".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Check if an expression is an `expect(...)` call or a chain like
/// `expect(...).not`.
fn is_expect_chain(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::CallExpression(call)) => {
            matches!(ctx.node(call.callee), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect")
        }
        Some(AstNode::StaticMemberExpression(member)) => is_expect_chain(member.object, ctx),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferStrictEqual)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_to_equal() {
        let diags = lint("expect(result).toEqual({ a: 1 });");
        assert_eq!(
            diags.len(),
            1,
            "`toEqual` should be flagged in favor of `toStrictEqual`"
        );
    }

    #[test]
    fn test_flags_to_equal_with_not() {
        let diags = lint("expect(result).not.toEqual({ a: 1 });");
        assert_eq!(diags.len(), 1, "`.not.toEqual` should also be flagged");
    }

    #[test]
    fn test_allows_to_strict_equal() {
        let diags = lint("expect(result).toStrictEqual({ a: 1 });");
        assert!(diags.is_empty(), "`toStrictEqual` should not be flagged");
    }
}
