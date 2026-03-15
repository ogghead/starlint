//! Rule: `no-unnecessary-array-flat-depth`
//!
//! Flag `.flat(1)` calls since `1` is the default depth for
//! `Array.prototype.flat()`. Calling `.flat()` without an argument is
//! equivalent and more concise.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.flat(1)` calls where the depth argument is the default value.
#[derive(Debug)]
pub struct NoUnnecessaryArrayFlatDepth;

impl LintRule for NoUnnecessaryArrayFlatDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unnecessary-array-flat-depth".to_owned(),
            description: "Disallow passing the default depth `1` to `.flat()`".to_owned(),
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

        // Must be a `.flat()` call
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "flat" {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // The argument must be the numeric literal `1`
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        if is_numeric_one(first_arg_id, ctx) {
            let call_span = Span::new(call.span.start, call.span.end);
            // Get the span of the argument to remove it
            let Some(AstNode::NumericLiteral(num_lit)) = ctx.node(first_arg_id) else {
                return;
            };
            let arg_span = Span::new(num_lit.span.start, num_lit.span.end);
            ctx.report(Diagnostic {
                rule_name: "no-unnecessary-array-flat-depth".to_owned(),
                message: "Unnecessary depth argument — `.flat()` defaults to depth `1`".to_owned(),
                span: call_span,
                severity: Severity::Warning,
                help: Some("Remove the depth argument".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove depth argument".to_owned(),
                    edits: vec![Edit {
                        span: arg_span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if a node is the numeric literal `1`.
fn is_numeric_one(id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.node(id), Some(AstNode::NumericLiteral(n)) if (n.value - 1.0).abs() < f64::EPSILON)
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnnecessaryArrayFlatDepth);

    #[test]
    fn test_flags_flat_with_one() {
        let diags = lint("arr.flat(1);");
        assert_eq!(diags.len(), 1, "arr.flat(1) should be flagged");
    }

    #[test]
    fn test_allows_flat_without_argument() {
        let diags = lint("arr.flat();");
        assert!(diags.is_empty(), "arr.flat() should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_two() {
        let diags = lint("arr.flat(2);");
        assert!(diags.is_empty(), "arr.flat(2) should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_infinity() {
        let diags = lint("arr.flat(Infinity);");
        assert!(diags.is_empty(), "arr.flat(Infinity) should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_zero() {
        let diags = lint("arr.flat(0);");
        assert!(diags.is_empty(), "arr.flat(0) should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_variable() {
        let diags = lint("arr.flat(depth);");
        assert!(diags.is_empty(), "arr.flat(depth) should not be flagged");
    }
}
