//! Rule: `consistent-empty-array-spread`
//!
//! Flag spreading an empty array literal (`..[]`) inside an array expression.
//! Patterns like `[...arr, ...[]]` or `[...[]]` are useless — the empty
//! spread contributes no elements.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags spreading an empty array literal inside an array expression.
#[derive(Debug)]
pub struct ConsistentEmptyArraySpread;

impl LintRule for ConsistentEmptyArraySpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-empty-array-spread".to_owned(),
            description: "Disallow spreading an empty array literal".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrayExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ArrayExpression(arr) = node else {
            return;
        };

        let has_empty_spread = arr
            .elements
            .iter()
            .any(|&el_id| is_empty_array_spread(ctx, el_id));

        if !has_empty_spread {
            return;
        }

        // Build fix: reconstruct array without empty spreads
        let source = ctx.source_text();
        let non_empty: Vec<&str> = arr
            .elements
            .iter()
            .filter(|&&el_id| !is_empty_array_spread(ctx, el_id))
            .filter_map(|&el_id| {
                let s = ctx.node(el_id)?.span();
                source.get(s.start as usize..s.end as usize)
            })
            .collect();
        let replacement = format!("[{}]", non_empty.join(", "));

        ctx.report(Diagnostic {
            rule_name: "consistent-empty-array-spread".to_owned(),
            message: "Spreading an empty array literal is unnecessary".to_owned(),
            span: Span::new(arr.span.start, arr.span.end),
            severity: Severity::Warning,
            help: Some("Remove the empty array spread".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove empty array spread".to_owned(),
                edits: vec![Edit {
                    span: Span::new(arr.span.start, arr.span.end),
                    replacement,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Check whether a node is a spread element with an empty array argument.
fn is_empty_array_spread(ctx: &LintContext<'_>, id: NodeId) -> bool {
    let Some(AstNode::SpreadElement(spread)) = ctx.node(id) else {
        return false;
    };
    matches!(
        ctx.node(spread.argument),
        Some(AstNode::ArrayExpression(arr)) if arr.elements.is_empty()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentEmptyArraySpread)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_single_empty_spread() {
        let diags = lint("var x = [...[]];");
        assert_eq!(diags.len(), 1, "[...[]] should be flagged");
    }

    #[test]
    fn test_flags_empty_spread_with_other_elements() {
        let diags = lint("var x = [...arr, ...[]];");
        assert_eq!(diags.len(), 1, "[...arr, ...[]] should be flagged");
    }

    #[test]
    fn test_allows_non_empty_spread() {
        let diags = lint("var x = [...[1, 2]];");
        assert!(diags.is_empty(), "[...[1, 2]] should not be flagged");
    }

    #[test]
    fn test_allows_spread_variable() {
        let diags = lint("var x = [...arr];");
        assert!(diags.is_empty(), "[...arr] should not be flagged");
    }

    #[test]
    fn test_allows_empty_array() {
        let diags = lint("var x = [];");
        assert!(diags.is_empty(), "empty array should not be flagged");
    }
}
