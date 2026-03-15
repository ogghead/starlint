//! Rule: `prefer-array-some` (unicorn)
//!
//! Prefer `.some()` over `.find()` when only checking for existence.
//! Using `.some()` returns a boolean directly and is more semantically
//! correct when you don't need the found element.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.find()` used in boolean contexts.
#[derive(Debug)]
pub struct PreferArraySome;

impl LintRule for PreferArraySome {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-some".to_owned(),
            description: "Prefer .some() over .find() for existence checks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Look for `if (arr.find(...))` — find used in boolean context
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        if let Some(prop_span) = find_property_span(if_stmt.test, ctx) {
            let test_span = ctx.node(if_stmt.test).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            ctx.report(Diagnostic {
                rule_name: "prefer-array-some".to_owned(),
                message: "Prefer `.some()` over `.find()` when checking for existence".to_owned(),
                span: Span::new(test_span.start, test_span.end),
                severity: Severity::Warning,
                help: Some("Replace `.find()` with `.some()`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace `.find()` with `.some()`".to_owned(),
                    edits: vec![Edit {
                        span: prop_span,
                        replacement: "some".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a `.find(...)` call and return the property name span.
fn find_property_span(expr_id: NodeId, ctx: &LintContext<'_>) -> Option<Span> {
    let AstNode::CallExpression(call) = ctx.node(expr_id)? else {
        return None;
    };

    let AstNode::StaticMemberExpression(member) = ctx.node(call.callee)? else {
        return None;
    };

    if member.property != "find" {
        return None;
    }

    // Find "find" in source text after the object
    let source = ctx.source_text();
    let obj_span = ctx.node(member.object).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );
    let search_start = usize::try_from(obj_span.end).unwrap_or(0);
    let prop_start = source
        .get(search_start..)
        .and_then(|s| s.find("find"))
        .map(|offset| u32::try_from(search_start.saturating_add(offset)).unwrap_or(0))?;
    let prop_end = prop_start.saturating_add(4); // "find" is 4 chars
    Some(Span::new(prop_start, prop_end))
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferArraySome);

    #[test]
    fn test_flags_find_in_if() {
        let diags = lint("if (arr.find(x => x > 0)) { }");
        assert_eq!(diags.len(), 1, "find in if condition should be flagged");
    }

    #[test]
    fn test_allows_some() {
        let diags = lint("if (arr.some(x => x > 0)) { }");
        assert!(diags.is_empty(), "some should not be flagged");
    }

    #[test]
    fn test_allows_find_in_assignment() {
        let diags = lint("var item = arr.find(x => x > 0);");
        assert!(diags.is_empty(), "find in assignment should not be flagged");
    }
}
