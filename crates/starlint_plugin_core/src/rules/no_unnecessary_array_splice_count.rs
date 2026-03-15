//! Rule: `no-unnecessary-array-splice-count`
//!
//! Flag `.splice(index, arr.length)` where the count argument is the array's
//! `.length`. Since `splice(index)` removes all remaining elements from the
//! given index, passing `.length` as the delete count is redundant.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.splice(index, obj.length)` where the count argument is redundant.
#[derive(Debug)]
pub struct NoUnnecessaryArraySpliceCount;

impl LintRule for NoUnnecessaryArraySpliceCount {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unnecessary-array-splice-count".to_owned(),
            description: "Disallow redundant `.length` as second argument to `.splice()`"
                .to_owned(),
            category: Category::Suggestion,
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

        // Must be a `.splice()` call
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "splice" {
            return;
        }

        // Must have exactly two arguments (index, count)
        // If there are more arguments (replacement elements), the `.length`
        // count is meaningful because it controls how many elements are removed
        // before inserting replacements, so we only flag the two-argument form.
        if call.arguments.len() != 2 {
            return;
        }

        // Second argument must be a `.length` member expression
        let Some(second_arg_id) = call.arguments.get(1) else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(length_member)) = ctx.node(*second_arg_id) else {
            return;
        };

        if length_member.property.as_str() != "length" {
            return;
        }

        // Compare the source text of the splice receiver and the .length owner
        let receiver_span = ctx.node(member.object).map_or(
            starlint_ast::types::Span::EMPTY,
            starlint_ast::AstNode::span,
        );
        let length_owner_span = ctx.node(length_member.object).map_or(
            starlint_ast::types::Span::EMPTY,
            starlint_ast::AstNode::span,
        );

        let receiver_start = usize::try_from(receiver_span.start).unwrap_or(0);
        let receiver_end = usize::try_from(receiver_span.end).unwrap_or(0);
        let owner_start = usize::try_from(length_owner_span.start).unwrap_or(0);
        let owner_end = usize::try_from(length_owner_span.end).unwrap_or(0);

        let source = ctx.source_text();
        let receiver_text = source.get(receiver_start..receiver_end);
        let owner_text = source.get(owner_start..owner_end);

        if let (Some(receiver), Some(owner)) = (receiver_text, owner_text) {
            if !receiver.is_empty() && receiver == owner {
                let call_span = Span::new(call.span.start, call.span.end);
                // Remove from end of first argument to end of second argument
                // This removes ", arr.length" from ".splice(0, arr.length)"
                let first_arg_end = call
                    .arguments
                    .first()
                    .map_or(0, |a| ctx.node(*a).map_or(0, |n| n.span().end));
                let second_arg_end = ctx.node(*second_arg_id).map_or(0, |n| n.span().end);
                let remove_span = Span::new(first_arg_end, second_arg_end);
                ctx.report(Diagnostic {
                    rule_name: "no-unnecessary-array-splice-count".to_owned(),
                    message: format!(
                        "Unnecessary `.length` argument — `{receiver}.splice(index)` already removes all remaining elements"
                    ),
                    span: call_span,
                    severity: Severity::Warning,
                    help: Some("Remove the `.length` count argument".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove `.length` count argument".to_owned(),
                        edits: vec![Edit {
                            span: remove_span,
                            replacement: String::new(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnnecessaryArraySpliceCount);

    #[test]
    fn test_flags_splice_with_length() {
        let diags = lint("arr.splice(0, arr.length);");
        assert_eq!(
            diags.len(),
            1,
            "arr.splice(0, arr.length) should be flagged"
        );
    }

    #[test]
    fn test_allows_splice_without_count() {
        let diags = lint("arr.splice(0);");
        assert!(diags.is_empty(), "arr.splice(0) should not be flagged");
    }

    #[test]
    fn test_allows_splice_with_numeric_count() {
        let diags = lint("arr.splice(0, 3);");
        assert!(diags.is_empty(), "arr.splice(0, 3) should not be flagged");
    }

    #[test]
    fn test_allows_different_objects() {
        let diags = lint("arr.splice(0, other.length);");
        assert!(diags.is_empty(), "different objects should not be flagged");
    }

    #[test]
    fn test_allows_splice_with_replacements() {
        let diags = lint("arr.splice(0, arr.length, 'a', 'b');");
        assert!(
            diags.is_empty(),
            "splice with replacement elements should not be flagged"
        );
    }
}
