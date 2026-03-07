//! Rule: `typescript/no-array-delete`
//!
//! Disallow using `delete` on array elements. Using `delete` on an array
//! creates a sparse array with a hole at that index, which is almost always
//! a bug. The length of the array is not updated and the element becomes
//! `undefined`. Use `Array.prototype.splice` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

/// Flags `delete arr[i]` expressions where the index is numeric, indicating
/// deletion from an array rather than an object.
#[derive(Debug)]
pub struct NoArrayDelete;

impl LintRule for NoArrayDelete {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-array-delete".to_owned(),
            description: "Disallow using `delete` on array elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::UnaryExpression(expr) = node else {
            return;
        };

        if expr.operator != UnaryOperator::Delete {
            return;
        }

        // Only flag computed member expressions (bracket access) where the
        // index expression looks numeric — this distinguishes array element
        // deletion from dynamic object key deletion.
        let Some(AstNode::ComputedMemberExpression(member)) = ctx.node(expr.argument) else {
            return;
        };

        if is_numeric_index(member.expression, ctx) {
            // Fix: `delete arr[i]` -> `arr.splice(i, 1)`
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let obj_span = ctx.node(member.object).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );
                let idx_span = ctx.node(member.expression).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );
                let obj_text = source
                    .get(obj_span.start as usize..obj_span.end as usize)
                    .unwrap_or("");
                let idx_text = source
                    .get(idx_span.start as usize..idx_span.end as usize)
                    .unwrap_or("");
                let replacement = format!("{obj_text}.splice({idx_text}, 1)");
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                })
            };

            ctx.report(Diagnostic {
                rule_name: "typescript/no-array-delete".to_owned(),
                message: "Do not `delete` array elements — it creates a sparse array hole. Use `splice` instead".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Error,
                help: Some("Use `.splice(index, 1)` to remove the element".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check whether an expression looks like a numeric array index.
///
/// Returns `true` for numeric literals (`delete arr[0]`) and identifiers
/// commonly used as loop counters (`delete arr[i]`), which strongly suggest
/// array element deletion rather than object property deletion.
fn is_numeric_index(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.node(expr_id),
        // A bare identifier as index (e.g. `delete arr[i]`) is likely an
        // array index from a loop — flag conservatively.
        Some(AstNode::NumericLiteral(_) | AstNode::IdentifierReference(_))
    )
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoArrayDelete)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_delete_with_numeric_index() {
        let diags = lint("delete arr[0];");
        assert_eq!(
            diags.len(),
            1,
            "delete with numeric index should be flagged"
        );
    }

    #[test]
    fn test_flags_delete_with_variable_index() {
        let diags = lint("delete arr[i];");
        assert_eq!(
            diags.len(),
            1,
            "delete with variable index should be flagged"
        );
    }

    #[test]
    fn test_allows_delete_with_string_key() {
        let diags = lint("delete obj[\"key\"];");
        assert!(
            diags.is_empty(),
            "delete with string key should not be flagged"
        );
    }

    #[test]
    fn test_allows_delete_with_static_property() {
        let diags = lint("delete obj.prop;");
        assert!(
            diags.is_empty(),
            "delete with static property access should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_delete_array_access() {
        let diags = lint("let x = arr[0];");
        assert!(
            diags.is_empty(),
            "non-delete array access should not be flagged"
        );
    }
}
