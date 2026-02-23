//! Rule: `no-length-as-slice-end`
//!
//! Flag `.slice(start, X.length)` calls where the second argument is a
//! `.length` member access. When `.length` is used as the end argument,
//! it is equivalent to omitting the second argument entirely since
//! `.slice()` defaults to slicing to the end.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.slice(start, X.length)` patterns where `.length` is redundant.
#[derive(Debug)]
pub struct NoLengthAsSliceEnd;

impl LintRule for NoLengthAsSliceEnd {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-length-as-slice-end".to_owned(),
            description: "Disallow using `.length` as the end argument in `.slice()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `.slice(...)` member call
        let Some(AstNode::StaticMemberExpression(slice_member)) = ctx.node(call.callee) else {
            return;
        };

        if slice_member.property.as_str() != "slice" {
            return;
        }

        // Must have exactly 2 arguments
        if call.arguments.len() != 2 {
            return;
        }

        let second_arg_id = call.arguments[1];
        let Some(second_arg) = ctx.node(second_arg_id) else {
            return;
        };

        // Check if the second argument is a `.length` member access
        let AstNode::StaticMemberExpression(length_member) = second_arg else {
            return;
        };
        if length_member.property.as_str() != "length" {
            return;
        }

        // Extract the receiver of `.slice()` and the object of `.length`
        // to check if they refer to the same entity
        let source = ctx.source_text();
        let slice_receiver_text = extract_source_text_by_id(ctx, slice_member.object, source);
        let length_object_text = extract_source_text_by_id(ctx, length_member.object, source);

        if let (Some(receiver), Some(length_obj)) = (slice_receiver_text, length_object_text) {
            if receiver == length_obj {
                let call_span = Span::new(call.span.start, call.span.end);
                // Remove from end of first argument to end of second argument
                let first_arg_span = ctx.node(call.arguments[0]).map(starlint_ast::AstNode::span);
                let first_arg_end = first_arg_span.map_or(0, |s| s.end);
                let second_arg_end = second_arg.span().end;
                let remove_span = Span::new(first_arg_end, second_arg_end);
                ctx.report(Diagnostic {
                    rule_name: "no-length-as-slice-end".to_owned(),
                    message: "Unnecessary `.length` as `.slice()` end — `.slice()` already defaults to the full length".to_owned(),
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

/// Extract source text for a node by its ID.
fn extract_source_text_by_id<'a>(
    ctx: &LintContext<'_>,
    id: NodeId,
    source: &'a str,
) -> Option<&'a str> {
    let node = ctx.node(id)?;
    let span = node.span();
    let start = usize::try_from(span.start).unwrap_or(0);
    let end = usize::try_from(span.end).unwrap_or(0);
    source.get(start..end)
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoLengthAsSliceEnd)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_slice_with_same_length() {
        let diags = lint("s.slice(0, s.length);");
        assert_eq!(
            diags.len(),
            1,
            "s.slice(0, s.length) should be flagged (same receiver)"
        );
    }

    #[test]
    fn test_flags_array_slice_with_same_length() {
        let diags = lint("arr.slice(1, arr.length);");
        assert_eq!(
            diags.len(),
            1,
            "arr.slice(1, arr.length) should be flagged (same receiver)"
        );
    }

    #[test]
    fn test_allows_slice_no_end() {
        let diags = lint("s.slice(0);");
        assert!(
            diags.is_empty(),
            "s.slice(0) should not be flagged (no end argument)"
        );
    }

    #[test]
    fn test_allows_slice_with_different_object() {
        let diags = lint("a.slice(1, b.length);");
        assert!(
            diags.is_empty(),
            "a.slice(1, b.length) should not be flagged (different objects)"
        );
    }

    #[test]
    fn test_allows_slice_with_numeric_end() {
        let diags = lint("s.slice(0, 5);");
        assert!(
            diags.is_empty(),
            "s.slice(0, 5) should not be flagged (numeric end)"
        );
    }

    #[test]
    fn test_allows_non_slice_call() {
        let diags = lint("s.substring(0, s.length);");
        assert!(
            diags.is_empty(),
            "substring calls should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_slice_with_three_args() {
        // .slice() only takes two args, but if somehow called with more,
        // we should not flag it
        let diags = lint("s.slice(0, s.length, extra);");
        assert!(
            diags.is_empty(),
            "slice with three args should not be flagged"
        );
    }
}
