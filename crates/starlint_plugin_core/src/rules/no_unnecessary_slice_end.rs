//! Rule: `no-unnecessary-slice-end`
//!
//! Flag `.slice(start, arr.length)` calls where the end argument equals the
//! receiver's `.length`. Since `slice(start)` already copies to the end of the
//! array/string, passing `.length` as the second argument is redundant.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.slice(start, obj.length)` where the end argument is redundant.
#[derive(Debug)]
pub struct NoUnnecessarySliceEnd;

impl LintRule for NoUnnecessarySliceEnd {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unnecessary-slice-end".to_owned(),
            description: "Disallow redundant `.length` as second argument to `.slice()`".to_owned(),
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

        // Must be a `.slice()` call
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "slice" {
            return;
        }

        // Must have exactly two arguments
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

        // Compare the source text of the slice receiver object and the .length owner
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
                // This removes ", arr.length" from ".slice(0, arr.length)"
                let first_arg_end = call
                    .arguments
                    .first()
                    .map_or(0, |a| ctx.node(*a).map_or(0, |n| n.span().end));
                let second_arg_end = ctx.node(*second_arg_id).map_or(0, |n| n.span().end);
                let remove_span = Span::new(first_arg_end, second_arg_end);
                ctx.report(Diagnostic {
                    rule_name: "no-unnecessary-slice-end".to_owned(),
                    message: format!(
                        "Unnecessary `.length` argument — `{receiver}.slice(start)` already copies to the end"
                    ),
                    span: call_span,
                    severity: Severity::Warning,
                    help: Some("Remove the `.length` end argument".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove `.length` end argument".to_owned(),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnnecessarySliceEnd)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_arr_slice_with_length() {
        let diags = lint("arr.slice(0, arr.length);");
        assert_eq!(diags.len(), 1, "arr.slice(0, arr.length) should be flagged");
    }

    #[test]
    fn test_flags_str_slice_with_length() {
        let diags = lint("str.slice(1, str.length);");
        assert_eq!(diags.len(), 1, "str.slice(1, str.length) should be flagged");
    }

    #[test]
    fn test_allows_slice_without_end() {
        let diags = lint("arr.slice(0);");
        assert!(diags.is_empty(), "arr.slice(0) should not be flagged");
    }

    #[test]
    fn test_allows_slice_with_numeric_end() {
        let diags = lint("arr.slice(0, 5);");
        assert!(diags.is_empty(), "arr.slice(0, 5) should not be flagged");
    }

    #[test]
    fn test_allows_different_objects() {
        let diags = lint("arr.slice(0, other.length);");
        assert!(diags.is_empty(), "different objects should not be flagged");
    }

    #[test]
    fn test_allows_no_arguments() {
        let diags = lint("arr.slice();");
        assert!(diags.is_empty(), "arr.slice() should not be flagged");
    }
}
