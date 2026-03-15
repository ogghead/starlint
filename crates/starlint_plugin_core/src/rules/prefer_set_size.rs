//! Rule: `prefer-set-size`
//!
//! Prefer `Set#size` over converting a Set to an array and checking `.length`.
//! Patterns like `[...set].length` or `Array.from(set).length` create an
//! unnecessary intermediate array just to count elements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::fix_utils::source_text_for_span;
use starlint_rule_framework::{FixBuilder, LintContext, LintRule};

/// Flags `.length` access on patterns that convert a Set to an array.
#[derive(Debug)]
pub struct PreferSetSize;

impl LintRule for PreferSetSize {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-set-size".to_owned(),
            description: "Prefer `Set#size` over converting to array and checking `.length`"
                .to_owned(),
            category: Category::Suggestion,
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

        if member.property.as_str() != "length" {
            return;
        }

        // Pattern 1: `[...x].length` — array with a single spread element
        if let Some(set_name) = get_spread_arg_name(member.object, ctx.source_text(), ctx) {
            let replacement = format!("{set_name}.size");
            ctx.report(Diagnostic {
                rule_name: "prefer-set-size".to_owned(),
                message: "Use `Set#size` instead of spreading into an array and checking `.length`"
                    .to_owned(),
                span: Span::new(member.span.start, member.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: FixBuilder::new(format!("Replace with `{replacement}`"), FixKind::SafeFix)
                    .replace(Span::new(member.span.start, member.span.end), replacement)
                    .build(),
                labels: vec![],
            });
            return;
        }

        // Pattern 2: `Array.from(x).length` — call to Array.from with one argument
        if let Some(set_name) = get_array_from_arg_name(member.object, ctx.source_text(), ctx) {
            let replacement = format!("{set_name}.size");
            ctx.report(Diagnostic {
                rule_name: "prefer-set-size".to_owned(),
                message: "Use `Set#size` instead of `Array.from()` and `.length`".to_owned(),
                span: Span::new(member.span.start, member.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: FixBuilder::new(format!("Replace with `{replacement}`"), FixKind::SafeFix)
                    .replace(Span::new(member.span.start, member.span.end), replacement)
                    .build(),
                labels: vec![],
            });
        }
    }
}

/// Extract the spread argument name from `[...something]` (array with a single spread element).
fn get_spread_arg_name<'s>(
    obj_id: NodeId,
    source: &'s str,
    ctx: &LintContext<'_>,
) -> Option<&'s str> {
    let AstNode::ArrayExpression(array) = ctx.node(obj_id)? else {
        return None;
    };

    if array.elements.len() != 1 {
        return None;
    }

    let elem_id = array.elements.first()?;
    let AstNode::SpreadElement(spread) = ctx.node(*elem_id)? else {
        return None;
    };

    let arg_span = ctx.node(spread.argument)?.span();
    source_text_for_span(source, Span::new(arg_span.start, arg_span.end))
}

/// Extract the argument name from `Array.from(something)` (single-argument call).
fn get_array_from_arg_name<'s>(
    obj_id: NodeId,
    source: &'s str,
    ctx: &LintContext<'_>,
) -> Option<&'s str> {
    let AstNode::CallExpression(call) = ctx.node(obj_id)? else {
        return None;
    };

    if call.arguments.len() != 1 {
        return None;
    }

    let AstNode::StaticMemberExpression(member) = ctx.node(call.callee)? else {
        return None;
    };

    if member.property.as_str() != "from" {
        return None;
    }

    if !matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Array")
    {
        return None;
    }

    let arg_id = call.arguments.first()?;
    let arg_span = ctx.node(*arg_id)?.span();
    source_text_for_span(source, Span::new(arg_span.start, arg_span.end))
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferSetSize);

    #[test]
    fn test_flags_spread_into_array_length() {
        let diags = lint("var n = [...mySet].length;");
        assert_eq!(diags.len(), 1, "[...mySet].length should be flagged");
    }

    #[test]
    fn test_flags_array_from_length() {
        let diags = lint("var n = Array.from(mySet).length;");
        assert_eq!(diags.len(), 1, "Array.from(mySet).length should be flagged");
    }

    #[test]
    fn test_allows_set_size() {
        let diags = lint("var n = mySet.size;");
        assert!(diags.is_empty(), "mySet.size should not be flagged");
    }

    #[test]
    fn test_allows_array_length() {
        let diags = lint("var n = myArray.length;");
        assert!(diags.is_empty(), "myArray.length should not be flagged");
    }

    #[test]
    fn test_allows_array_from_with_mapper() {
        let diags = lint("var n = Array.from(mySet, x => x * 2).length;");
        assert!(
            diags.is_empty(),
            "Array.from with mapper should not be flagged"
        );
    }

    #[test]
    fn test_allows_multi_element_spread() {
        let diags = lint("var n = [1, ...mySet].length;");
        assert!(
            diags.is_empty(),
            "array with multiple elements should not be flagged"
        );
    }
}
