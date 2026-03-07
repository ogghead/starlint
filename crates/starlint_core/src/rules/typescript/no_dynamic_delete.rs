//! Rule: `typescript/no-dynamic-delete`
//!
//! Disallow `delete` with computed key expressions. Using `delete` with a
//! dynamic (bracket-accessed) key makes code harder to reason about and
//! prevents certain engine optimizations. Use `Map` or `Set` for dynamic
//! key collections, or `Reflect.deleteProperty` for explicit intent.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

/// Flags `delete` expressions that use computed (bracket) member access.
#[derive(Debug)]
pub struct NoDynamicDelete;

impl LintRule for NoDynamicDelete {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-dynamic-delete".to_owned(),
            description: "Disallow `delete` with computed key expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

        // Only flag when the operand is a computed member expression (bracket access).
        // `delete obj.prop` (static access) is fine.
        if let Some(AstNode::ComputedMemberExpression(computed)) = ctx.node(expr.argument) {
            // Fix: delete obj[key] → Reflect.deleteProperty(obj, key)
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let obj_span = ctx.node(computed.object).map_or(Span::new(0, 0), |n| {
                    let s = n.span();
                    Span::new(s.start, s.end)
                });
                let key_span = ctx.node(computed.expression).map_or(Span::new(0, 0), |n| {
                    let s = n.span();
                    Span::new(s.start, s.end)
                });
                let obj_text = source.get(obj_span.start as usize..obj_span.end as usize);
                let key_text = source.get(key_span.start as usize..key_span.end as usize);
                match (obj_text, key_text) {
                    (Some(obj), Some(key)) => {
                        let replacement = format!("Reflect.deleteProperty({obj}, {key})");
                        Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        })
                    }
                    _ => None,
                }
            };

            ctx.report(Diagnostic {
                rule_name: "typescript/no-dynamic-delete".to_owned(),
                message: "Do not `delete` dynamically computed keys — use `Map` or `Set` instead"
                    .to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDynamicDelete)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_delete_with_variable_key() {
        let diags = lint("delete obj[key];");
        assert_eq!(diags.len(), 1, "delete with dynamic key should be flagged");
    }

    #[test]
    fn test_flags_delete_with_string_key() {
        let diags = lint("delete obj[\"key\"];");
        assert_eq!(
            diags.len(),
            1,
            "delete with string bracket key should be flagged"
        );
    }

    #[test]
    fn test_allows_delete_with_static_property() {
        let diags = lint("delete obj.key;");
        assert!(
            diags.is_empty(),
            "delete with static property access should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_delete_computed_access() {
        let diags = lint("obj[key];");
        assert!(
            diags.is_empty(),
            "non-delete computed access should not be flagged"
        );
    }
}
