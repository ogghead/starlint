//! Rule: `prefer-negative-index` (unicorn)
//!
//! Prefer negative index over `.length - index` for methods that support it.
//! Methods like `.slice()`, `.at()`, `.splice()` accept negative indices.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.slice(arr.length - N)` and similar patterns.
#[derive(Debug)]
pub struct PreferNegativeIndex;

/// Methods that accept negative indices.
const NEGATIVE_INDEX_METHODS: &[&str] = &["slice", "splice", "at", "with"];

impl LintRule for PreferNegativeIndex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-negative-index".to_owned(),
            description: "Prefer negative index over .length - index".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `something.method(something.length - N)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();
        if !NEGATIVE_INDEX_METHODS.contains(&method_name) {
            return;
        }

        let member_object = member.object;

        // Check arguments for `.length - N` pattern
        for arg_id in &call.arguments {
            let Some(AstNode::BinaryExpression(bin)) = ctx.node(*arg_id) else {
                continue;
            };

            if !matches!(bin.operator, BinaryOperator::Subtraction) {
                continue;
            }

            // Left side should be `something.length`
            let Some(AstNode::StaticMemberExpression(len_member)) = ctx.node(bin.left) else {
                continue;
            };

            if len_member.property != "length" {
                continue;
            }

            // Right side should be a numeric literal
            let Some(AstNode::NumericLiteral(num_lit)) = ctx.node(bin.right) else {
                continue;
            };

            let num_value = num_lit.value;
            let bin_span = bin.span;
            let len_member_object = len_member.object;

            // Check that the object and .length owner are the same identifier
            if let (
                Some(AstNode::IdentifierReference(obj_id)),
                Some(AstNode::IdentifierReference(len_obj_id)),
            ) = (ctx.node(member_object), ctx.node(len_member_object))
            {
                if obj_id.name == len_obj_id.name {
                    let n = num_value;
                    #[allow(clippy::cast_possible_truncation)]
                    let neg_val = if (n - n.round()).abs() < f64::EPSILON {
                        format!("-{}", n as i64)
                    } else {
                        format!("-{n}")
                    };

                    ctx.report(Diagnostic {
                        rule_name: "prefer-negative-index".to_owned(),
                        message: format!(
                            "Use a negative index instead of `.length` subtraction in `.{method_name}()`"
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Warning,
                        help: Some(format!("Replace with `{neg_val}`")),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Replace `{obj_id}.length - {n}` with `{neg_val}`", obj_id = obj_id.name),
                            edits: vec![Edit {
                                span: Span::new(bin_span.start, bin_span.end),
                                replacement: neg_val,
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferNegativeIndex);

    #[test]
    fn test_flags_slice_length_minus() {
        let diags = lint("arr.slice(arr.length - 2);");
        assert_eq!(
            diags.len(),
            1,
            "arr.slice(arr.length - 2) should be flagged"
        );
    }

    #[test]
    fn test_allows_slice_negative() {
        let diags = lint("arr.slice(-2);");
        assert!(diags.is_empty(), "arr.slice(-2) should not be flagged");
    }

    #[test]
    fn test_allows_different_objects() {
        let diags = lint("arr.slice(other.length - 2);");
        assert!(diags.is_empty(), "different objects should not be flagged");
    }

    #[test]
    fn test_allows_non_negative_index_method() {
        let diags = lint("arr.push(arr.length - 1);");
        assert!(
            diags.is_empty(),
            "non-negative-index method should not be flagged"
        );
    }
}
