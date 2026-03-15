//! Rule: `prefer-array-find` (unicorn)
//!
//! Prefer `.find()` over `.filter()[0]`. When only the first matching
//! element is needed, `.find()` is more efficient and readable.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.filter(...)[0]` patterns that should use `.find()`.
#[derive(Debug)]
pub struct PreferArrayFind;

impl LintRule for PreferArrayFind {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-find".to_owned(),
            description: "Prefer .find() over .filter()[0]".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ComputedMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ComputedMemberExpression(computed) = node else {
            return;
        };

        // Check if the index is `0`
        let Some(AstNode::NumericLiteral(num)) = ctx.node(computed.expression) else {
            return;
        };

        if num.value != 0.0 {
            return;
        }

        // Check if the object is a `.filter(...)` call
        let Some(AstNode::CallExpression(call)) = ctx.node(computed.object) else {
            return;
        };

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property == "filter" {
            // Two edits: rename .filter -> .find, and delete [0]
            // We need to compute the span of "filter" from source text
            let source = ctx.source_text();
            let call_span = ctx.node(computed.object).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            // Find "filter" in the member expression source text
            // The property starts after the dot: object_end + 1
            let obj_span = ctx.node(member.object).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            // Look for "filter" in the source after the object span
            let search_start = usize::try_from(obj_span.end).unwrap_or(0);
            let prop_start = source
                .get(search_start..)
                .and_then(|s| s.find("filter"))
                .map(|offset| u32::try_from(search_start.saturating_add(offset)).unwrap_or(0));

            if let Some(prop_start_u32) = prop_start {
                let prop_end = prop_start_u32.saturating_add(6); // "filter" is 6 chars
                let prop_span = Span::new(prop_start_u32, prop_end);
                // Delete from end of call expression to end of computed member (the `[0]`)
                let call_end = call_span.end;
                let computed_end = computed.span.end;

                ctx.report(Diagnostic {
                    rule_name: "prefer-array-find".to_owned(),
                    message: "Prefer `.find()` over `.filter()[0]`".to_owned(),
                    span: Span::new(computed.span.start, computed.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace `.filter()[0]` with `.find()`".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Replace `.filter()[0]` with `.find()`".to_owned(),
                        edits: vec![
                            Edit {
                                span: prop_span,
                                replacement: "find".to_owned(),
                            },
                            Edit {
                                span: Span::new(call_end, computed_end),
                                replacement: String::new(),
                            },
                        ],
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

    starlint_rule_framework::lint_rule_test!(PreferArrayFind);

    #[test]
    fn test_flags_filter_zero() {
        let diags = lint("var x = arr.filter(fn)[0];");
        assert_eq!(diags.len(), 1, ".filter()[0] should be flagged");
    }

    #[test]
    fn test_allows_find() {
        let diags = lint("var x = arr.find(fn);");
        assert!(diags.is_empty(), ".find() should not be flagged");
    }

    #[test]
    fn test_allows_filter_non_zero() {
        let diags = lint("var x = arr.filter(fn)[1];");
        assert!(diags.is_empty(), ".filter()[1] should not be flagged");
    }

    #[test]
    fn test_allows_filter_variable() {
        let diags = lint("var x = arr.filter(fn);");
        assert!(
            diags.is_empty(),
            ".filter() without index should not be flagged"
        );
    }
}
