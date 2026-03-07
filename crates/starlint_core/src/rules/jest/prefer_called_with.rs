//! Rule: `jest/prefer-called-with`
//!
//! Suggest `toHaveBeenCalledWith` over `toHaveBeenCalled`. Using the more
//! specific `toHaveBeenCalledWith` ensures mock functions are called with
//! the expected arguments, catching bugs where the right function is called
//! but with wrong parameters.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `toHaveBeenCalled()` / `toBeCalled()` in favor of `toHaveBeenCalledWith()`.
#[derive(Debug)]
pub struct PreferCalledWith;

impl LintRule for PreferCalledWith {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-called-with".to_owned(),
            description: "Suggest using `toHaveBeenCalledWith()` over `toHaveBeenCalled()`"
                .to_owned(),
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
        let method = member.property.as_str();
        if method != "toHaveBeenCalled" && method != "toBeCalled" {
            return;
        }

        if !is_expect_chain(member.object, ctx) {
            return;
        }

        // Property is a String, not a node -- compute span from source text
        // The property span can be estimated from member span
        let source = ctx.source_text();
        let member_start = usize::try_from(member.span.start).unwrap_or(0);
        let member_end = usize::try_from(member.span.end).unwrap_or(0);
        let member_text = source.get(member_start..member_end).unwrap_or("");
        // Find where the property name starts in the member text (after the last `.`)
        let prop_offset = member_text.rfind('.').map_or(0, |i| i + 1);
        #[allow(clippy::as_conversions)]
        let prop_start = member.span.start + prop_offset as u32;
        #[allow(clippy::as_conversions)]
        let prop_end = prop_start + method.len() as u32;

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-called-with".to_owned(),
            message: format!(
                "Use `toHaveBeenCalledWith()` instead of `{method}()` for more specific assertions"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace with `toHaveBeenCalledWith`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(prop_start, prop_end),
                    replacement: "toHaveBeenCalledWith".to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferCalledWith)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_to_have_been_called() {
        let diags = lint("expect(mockFn).toHaveBeenCalled();");
        assert_eq!(diags.len(), 1, "`toHaveBeenCalled()` should be flagged");
    }

    #[test]
    fn test_flags_to_be_called() {
        let diags = lint("expect(mockFn).toBeCalled();");
        assert_eq!(diags.len(), 1, "`toBeCalled()` should be flagged");
    }

    #[test]
    fn test_allows_to_have_been_called_with() {
        let diags = lint("expect(mockFn).toHaveBeenCalledWith(1, 2);");
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledWith()` should not be flagged"
        );
    }
}
