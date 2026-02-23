//! Rule: `prefer-at` (unicorn)
//!
//! Prefer `.at()` for index access from the end of an array/string.
//! `array.at(-1)` is more readable than `array[array.length - 1]`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `arr[arr.length - 1]` patterns that should use `.at(-1)`.
#[derive(Debug)]
pub struct PreferAt;

impl LintRule for PreferAt {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-at".to_owned(),
            description: "Prefer `.at()` for index access from the end".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ComputedMemberExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ComputedMemberExpression(computed) = node else {
            return;
        };

        // Check for `obj[obj.length - N]` pattern
        let Some(AstNode::BinaryExpression(bin)) = ctx.node(computed.expression) else {
            return;
        };

        if !matches!(bin.operator, BinaryOperator::Subtraction) {
            return;
        }

        // Left side should be `something.length`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(bin.left) else {
            return;
        };

        if member.property != "length" {
            return;
        }

        // Right side should be a numeric literal
        let Some(AstNode::NumericLiteral(num_lit)) = ctx.node(bin.right) else {
            return;
        };

        let num_value = num_lit.value;
        let member_object = member.object;

        // The object being accessed and the `.length` owner should be the same
        if let (
            Some(AstNode::IdentifierReference(obj_id)),
            Some(AstNode::IdentifierReference(len_obj_id)),
        ) = (ctx.node(computed.object), ctx.node(member_object))
        {
            if obj_id.name == len_obj_id.name {
                let obj_name = obj_id.name.as_str();
                // Format the negative index value
                let n = num_value;
                #[allow(clippy::cast_possible_truncation)]
                let neg_index = if (n - n.round()).abs() < f64::EPSILON {
                    format!("-{}", n as i64)
                } else {
                    format!("-{n}")
                };
                let replacement = format!("{obj_name}.at({neg_index})");

                ctx.report(Diagnostic {
                    rule_name: "prefer-at".to_owned(),
                    message: "Prefer `.at()` for index access from the end of an array or string"
                        .to_owned(),
                    span: Span::new(computed.span.start, computed.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace with `{replacement}`")),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(computed.span.start, computed.span.end),
                            replacement,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferAt)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_length_minus_one() {
        let diags = lint("var x = arr[arr.length - 1];");
        assert_eq!(diags.len(), 1, "arr[arr.length - 1] should be flagged");
    }

    #[test]
    fn test_flags_length_minus_two() {
        let diags = lint("var x = arr[arr.length - 2];");
        assert_eq!(diags.len(), 1, "arr[arr.length - 2] should be flagged");
    }

    #[test]
    fn test_allows_at() {
        let diags = lint("var x = arr.at(-1);");
        assert!(diags.is_empty(), ".at(-1) should not be flagged");
    }

    #[test]
    fn test_allows_normal_index() {
        let diags = lint("var x = arr[0];");
        assert!(diags.is_empty(), "arr[0] should not be flagged");
    }

    #[test]
    fn test_allows_different_objects() {
        let diags = lint("var x = arr[other.length - 1];");
        assert!(diags.is_empty(), "different objects should not be flagged");
    }
}
